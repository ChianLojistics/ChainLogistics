#!/bin/bash
# ChainLogistics Certificate Renewal Script
# This script automates the renewal of SSL/TLS certificates using Certbot.

set -e

echo "Starting certificate renewal process..."

# Ensure we are running as root or have necessary permissions
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (or via sudo)"
   exit 1
fi

# Dry run by default if no arguments provided
DRY_RUN="--dry-run"
if [[ "$1" == "--force" ]]; then
    DRY_RUN=""
    echo "WARNING: Running in FORCE mode. Actual certificates will be requested."
fi

# Run Certbot
# Note: This assumes Nginx is running and can serve the .well-known challenge
certbot renew $DRY_RUN --webroot -w /var/www/certbot --post-hook "nginx -s reload"

echo "Renewal process completed."
