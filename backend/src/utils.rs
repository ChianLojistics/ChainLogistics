use std::time::Duration;
use sqlx::PgPool;
use chrono::{DateTime, Utc};
use crate::services::{SyncService, ProductService, EventService};

pub mod aggregation;
pub mod crypto;

pub struct BackupService {
    pool: PgPool,
}

impl BackupService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_backup(&self) -> Result<String, sqlx::Error> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("chainlogistics_backup_{}.sql", timestamp);
        
        // This would typically call pg_dump or use PostgreSQL's backup APIs
        // For now, we'll create a logical backup by exporting key tables
        let backup_path = format!("/backups/{}", backup_filename);
        
        // Create backup directory if it doesn't exist
        tokio::fs::create_dir_all("/backups").await
            .map_err(|e| sqlx::Error::Io(e.into()))?;

        // Export products
        let products = sqlx::query!("SELECT * FROM products")
            .fetch_all(&self.pool)
            .await?;

        // Export events
        let events = sqlx::query!("SELECT * FROM tracking_events")
            .fetch_all(&self.pool)
            .await?;

        // Write backup file (simplified - in production use pg_dump)
        let backup_content = format!(
            "-- ChainLogistics Backup - {}\n\n-- Products\n{:?}\n\n-- Events\n{:?}\n",
            timestamp, products, events
        );

        tokio::fs::write(&backup_path, backup_content).await
            .map_err(|e| sqlx::Error::Io(e.into()))?;

        tracing::info!("Backup created: {}", backup_path);
        Ok(backup_path)
    }

    pub async fn restore_backup(&self, backup_path: &str) -> Result<(), sqlx::Error> {
        // In production, this would use psql or pg_restore
        tracing::warn!("Restore functionality not implemented for path: {}", backup_path);
        Ok(())
    }

    pub async fn cleanup_old_backups(&self, retain_days: i64) -> Result<Vec<String>, sqlx::Error> {
        let mut dir = tokio::fs::read_dir("/backups").await
            .map_err(|e| sqlx::Error::Io(e.into()))?;

        let cutoff_time = Utc::now() - chrono::Duration::days(retain_days);
        let mut removed_files = Vec::new();

        while let Some(entry) = dir.next_entry().await
            .map_err(|e| sqlx::Error::Io(e.into))? {
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time: DateTime<Utc> = modified.into();
                        if modified_time < cutoff_time {
                            if tokio::fs::remove_file(&path).await.is_ok() {
                                removed_files.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("Cleaned up {} old backup files", removed_files.len());
        Ok(removed_files)
    }
}

// Cron service for scheduled tasks
pub struct CronService {
    pool: PgPool,
    redis_client: redis::Client,
    backup_service: BackupService,
    sync_service: SyncService,
}

impl CronService {
    pub fn new(pool: PgPool, redis_client: redis::Client) -> Self {
        Self {
            pool: pool.clone(),
            redis_client: redis_client.clone(),
            backup_service: BackupService::new(pool.clone()),
            sync_service: SyncService::new(pool, redis_client),
        }
    }

    pub async fn start_scheduler(&self) {
        let pool = self.pool.clone();
        let backup_service = self.backup_service.clone();
        
        // Daily backup at 2 AM UTC
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(86400)); // 24 hours
            
            loop {
                interval.tick().await;
                
                // Check if it's 2 AM UTC (simplified - in production use a proper cron library)
                let now = Utc::now();
                if now.hour() == 2 && now.minute() == 0 {
                    if let Err(e) = backup_service.create_backup().await {
                        tracing::error!("Failed to create backup: {}", e);
                    }
                    
                    // Clean up backups older than 30 days
                    if let Err(e) = backup_service.cleanup_old_backups(30).await {
                        tracing::error!("Failed to cleanup old backups: {}", e);
                    }
                }
            }
        });

        // Sync with smart contracts every 5 minutes
        let sync_service = self.sync_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                // In a real implementation, this would:
                // 1. Query the smart contract for new products/events
                // 2. Sync them to the database
                tracing::debug!("Running scheduled sync with smart contracts");
                
                // Placeholder for sync logic
                // sync_service.sync_from_contract().await;
            }
        });

        tracing::info!("Cron scheduler started");
    }
}

// Clone implementations for async tasks
impl Clone for BackupService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl Clone for SyncService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            redis_client: self.redis_client.clone(),
            product_service: ProductService::new(self.pool.clone(), self.redis_client.clone()),
            event_service: EventService::new(self.pool.clone(), self.redis_client.clone()),
        }
    }
}