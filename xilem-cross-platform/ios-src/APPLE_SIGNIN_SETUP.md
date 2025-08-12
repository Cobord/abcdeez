# Apple Sign In Setup Guide

## Server-to-Server Notifications

Apple Sign in with Apple requires setting up server-to-server notifications to handle important account events.

### Events You'll Receive

1. **Email Disabled/Enabled** - User changes email forwarding preferences
2. **Consent Revoked** - User stops using Sign in with Apple with your app  
3. **Account Delete** - User permanently deletes their Apple Account (requires action within 30 days)

### Setup Steps

1. **Configure Webhook URL in Apple Developer Console**
   - Go to [Apple Developer Console](https://developer.apple.com)
   - Navigate to your app's Sign in with Apple configuration
   - Add your server notification URL: `https://your-domain.com/auth/apple/notifications`
   - URL must use HTTPS with TLS 1.2 or higher

2. **Implement the Webhook Handler**
   - See `/web-backend/src/auth/apple_notifications.rs` for the implementation
   - Verify JWT signatures using Apple's public keys
   - Handle each event type appropriately

3. **Fetch Apple's Public Keys**
   ```rust
   // Fetch from: https://appleid.apple.com/auth/keys
   // Cache these keys and refresh daily
   ```

4. **Database Schema Requirements**
   ```sql
   -- Track Apple Sign In users
   CREATE TABLE apple_users (
       user_id VARCHAR(255) PRIMARY KEY,
       email VARCHAR(255),
       email_enabled BOOLEAN DEFAULT true,
       consent_status VARCHAR(50) DEFAULT 'active',
       deletion_scheduled_at TIMESTAMP NULL,
       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
       updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
   );
   ```

### Compliance Requirements

#### Account Deletion (CRITICAL)
When you receive an `account-delete` event:
- **Must delete user data within 30 days**
- Remove all personal information
- Log the deletion for compliance
- Cannot be reversed

#### Consent Revoked
When you receive a `consent-revoked` event:
- Stop using their Apple-provided data
- Offer alternative sign-in methods
- Update user's authentication status

#### Email Status Changes
When email forwarding is disabled:
- Stop sending emails to that address immediately
- Mark email as unreachable in your system
- Can resume when `email-enabled` event is received

### Testing

1. **Test in Sandbox Environment**
   - Use Apple's sandbox environment first
   - Verify JWT signature validation works
   - Test each event type handling

2. **Production Checklist**
   - [ ] HTTPS with TLS 1.2+ configured
   - [ ] JWT verification implemented
   - [ ] All event types handled
   - [ ] Database updates working
   - [ ] Deletion compliance process in place
   - [ ] Error logging configured
   - [ ] Monitoring/alerting set up

### Security Considerations

1. **Always verify JWT signatures** - Never trust unverified payloads
2. **Use HTTPS only** - Apple requires TLS 1.2 or higher
3. **Validate audience claim** - Ensure the notification is for your app
4. **Log all events** - Maintain audit trail for compliance
5. **Implement idempotency** - Handle duplicate notifications gracefully

### Monitoring

Set up alerts for:
- Failed JWT verifications
- Account deletion events (compliance critical)
- High error rates
- Endpoint availability

### Resources

- [Apple Documentation](https://developer.apple.com/documentation/sign_in_with_apple/processing_changes_for_sign_in_with_apple_accounts)
- [JWT Verification](https://developer.apple.com/documentation/sign_in_with_apple/fetch_apple_s_public_key_for_verifying_token_signature)
- [REST API Reference](https://developer.apple.com/documentation/sign_in_with_apple/sign_in_with_apple_rest_api)