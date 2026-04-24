# Security Implementation Guide

## Overview

Comprehensive security implementation for ChainLogistics including HTTPS enforcement, SSL/TLS configuration, and CORS policies.

## HTTPS Enforcement

### Configuration

```bash
ENFORCE_HTTPS=true
HSTS_MAX_AGE=31536000
TLS_ENABLED=true
TLS_CERT_PATH=/path/to/cert.pem
TLS_KEY_PATH=/path/to/key.pem
```

### Implementation

- HTTPS redirect middleware (301 Moved Permanently)
- HSTS headers with preload directive
- Support for load balancer SSL termination
- X-Forwarded-Proto header detection

## SSL/TLS Configuration

### Protocols

- TLS 1.3 (preferred)
- TLS 1.2 (minimum)
- SSLv3, TLS 1.0, TLS 1.1 (disabled)

### Certificate Management

- Use trusted CA (Let's Encrypt, DigiCert)
- Automate renewal with certbot
- Monitor expiry (alert 30 days before)
- Secure storage (not in version control)

## CORS Configuration

### Setup

```bash
ALLOWED_ORIGINS=https://app.chainlogistics.com,*.chainlogistics.com
```

### Features

- Origin whitelist validation
- Wildcard subdomain support
- Preflight request handling
- Credential support
- 24-hour preflight caching

## Security Headers

All responses include:

- Strict-Transport-Security (HSTS)
- Content-Security-Policy (CSP)
- X-Frame-Options: DENY
- X-Content-Type-Options: nosniff
- X-XSS-Protection
- Referrer-Policy
- Permissions-Policy

## Testing

```bash
# Test HTTPS redirect
curl -I http://api.chainlogistics.com

# Test CORS
curl -H "Origin: https://app.chainlogistics.com" \
     -X OPTIONS https://api.chainlogistics.com/api/products

# Check security headers
curl -I https://api.chainlogistics.com
```

## Compliance

- PCI DSS: TLS 1.2+, strong ciphers
- GDPR: Data encryption in transit
- SOC 2: Encryption standards, access logging

## Deployment Checklist

- [ ] Obtain SSL/TLS certificates
- [ ] Configure environment variables
- [ ] Test HTTPS enforcement
- [ ] Verify CORS policies
- [ ] Run SSL Labs scan (target: A+)
- [ ] Enable monitoring and alerting
