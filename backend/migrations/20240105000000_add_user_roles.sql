-- Add role column to users table
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'customer';
ALTER TABLE users DROP COLUMN is_admin;
