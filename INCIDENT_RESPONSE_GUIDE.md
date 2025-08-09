# 🚨 Incident Response Guide
## Graph Learning System Security & Operational Incident Response

---

## 📋 Quick Response Checklist

**IMMEDIATE ACTIONS (First 5 minutes):**
- [ ] Assess threat severity (Critical/High/Medium/Low)
- [ ] Alert incident response team
- [ ] Document incident start time and initial observations
- [ ] Take immediate containment actions if needed
- [ ] Begin evidence preservation

---

## 🎯 Incident Classification & Severity Levels

### **🔴 CRITICAL (P0) - Response Time: 15 minutes**
- Active data breach or compromise
- System completely down affecting all users
- Authentication bypass or admin panel compromise
- Payment system compromise
- Malicious code execution on servers

### **🟠 HIGH (P1) - Response Time: 1 hour**
- Partial service disruption affecting >50% users
- OAuth provider authentication failures
- Database corruption or significant data loss
- Successful privilege escalation
- DDoS attacks significantly impacting performance

### **🟡 MEDIUM (P2) - Response Time: 4 hours**
- Performance degradation affecting <50% users
- Non-critical security vulnerability discovered
- Failed deployment requiring rollback
- Individual user data exposure
- Background job failures affecting operations

### **🟢 LOW (P3) - Response Time: 24 hours**
- Minor bugs or features not working as expected
- Cosmetic issues or documentation problems
- Monitoring alerts for non-critical services
- Planned maintenance communication

---

## 🔍 Detection & Alerting Sources

### **Automated Monitoring**
```bash
# System Health Endpoints
curl https://abcdeez.fg-goose.online/health/live
curl https://abcdeez.fg-goose.online/health/ready
curl https://abcdeez.fg-goose.online/health/health

# Performance Metrics
curl https://abcdeez.fg-goose.online/health/metrics
curl https://abcdeez.fg-goose.online/health/performance
```

### **Security Event Detection**
- **Admin Panel Access**: Monitor `/admin` access patterns
- **Authentication Failures**: Track failed login attempts in audit logs
- **OAuth Issues**: Monitor Apple/GitHub authentication failures
- **Rate Limiting**: Excessive rate limit hits may indicate attack
- **IP Blocking**: Auto-blocked IPs in audit trail

### **Manual Monitoring Locations**
- GitHub Actions deployment status
- Server logs via SSH/containers
- Database connection and query performance
- User reports via support channels

---

## 🚨 Incident Response Procedures

### **Phase 1: Detection & Assessment (0-15 minutes)**

**1. Incident Identification**
```bash
# Check system status immediately
curl -s https://abcdeez.fg-goose.online/health/health | jq '.'

# Check recent deployments
gh run list --limit 5

# Review admin panel for alerts
# Access: https://abcdeez.fg-goose.online/admin
```

**2. Severity Assessment Matrix**
| Impact | User Facing | Data Integrity | Security | Severity |
|--------|-------------|----------------|----------|----------|
| All Users | Down | Compromised | Breached | P0 |
| >50% Users | Degraded | At Risk | Vulnerable | P1 |
| <50% Users | Minor Issues | Stable | Monitoring | P2 |
| Minimal | Cosmetic | Stable | Secure | P3 |

**3. Initial Response Team Notification**
```
Incident: [SEVERITY] - [Brief Description]
Time: [UTC Timestamp]
Affected: [Users/Systems/Data]
Reporter: [Name/System]
Initial Assessment: [1-2 sentences]
```

### **Phase 2: Containment (15-60 minutes)**

**Security Incidents**
```bash
# 1. Block malicious IPs (if identified)
# Add to IP blocklist via admin panel or direct cache

# 2. Disable compromised user accounts
sqlite3 local-dev/dev.db "UPDATE users SET metadata = json_set(metadata, '$.status', 'disabled') WHERE id = 'compromised-user-id';"

# 3. Revoke JWT sessions
# Add session IDs to blacklist via admin panel

# 4. Emergency admin access lockdown
sqlite3 production.db "UPDATE users SET metadata = json_set(metadata, '$.mfa_required', true) WHERE metadata->>'role' = 'admin';"
```

**Service Outage**
```bash
# 1. Check deployment status
gh run list --workflow=deploy-production.yml --limit 3

# 2. Emergency rollback if needed
git log --oneline -5 main
git revert [bad-commit-hash]
git push origin main  # Triggers auto-deployment

# 3. Switch to maintenance mode
curl -X PUT https://abcdeez.fg-goose.online/api/admin/config \
  -H "Authorization: Bearer YOUR_ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d '{"maintenance_mode": "true"}'
```

**Database Issues**
```bash
# 1. Check database health
curl -s https://abcdeez.fg-goose.online/health/health | jq '.database'

# 2. Create emergency backup
pg_dump $DATABASE_URL > emergency_backup_$(date +%Y%m%d_%H%M%S).sql

# 3. Switch to read-only mode if needed
# (Implementation depends on your database setup)
```

### **Phase 3: Investigation (Parallel to Containment)**

**Evidence Collection**
```bash
# 1. Capture system status
curl -s https://abcdeez.fg-goose.online/health/health > incident_health_$(date +%Y%m%d_%H%M%S).json

# 2. Export recent audit logs
curl -H "Authorization: Bearer $ADMIN_JWT" \
  "https://abcdeez.fg-goose.online/api/admin/audit?limit=500" > incident_audit_$(date +%Y%m%d_%H%M%S).json

# 3. Generate compliance audit report
curl -H "Authorization: Bearer $ADMIN_JWT" \
  "https://abcdeez.fg-goose.online/api/admin/audit-report" > incident_report_$(date +%Y%m%d_%H%M%S).json

# 4. Capture server logs
docker logs web-backend-container > incident_logs_$(date +%Y%m%d_%H%M%S).txt
```

**Analysis Questions**
- **When**: What time did the incident start? What changed recently?
- **What**: What specific systems/users/data are affected?
- **Who**: Which user accounts or IP addresses are involved?
- **How**: What was the attack vector or failure mechanism?
- **Why**: What vulnerability or process failure enabled this?

### **Phase 4: Eradication & Recovery**

**Security Incident Recovery**
1. **Patch Vulnerabilities**: Deploy fixes immediately
2. **Reset Credentials**: Force password resets for affected accounts
3. **Update Security Rules**: Add new detection rules
4. **Rebuild Compromised Systems**: If servers were compromised

**Service Recovery**
1. **Fix Root Cause**: Deploy corrected code
2. **Data Restoration**: Restore from backups if needed
3. **Gradual Service Restoration**: Bring services back online incrementally
4. **Performance Validation**: Confirm system stability

```bash
# Service recovery validation
curl -s https://abcdeez.fg-goose.online/health/performance | jq '.response_time_ms'

# User authentication test
curl -X POST https://abcdeez.fg-goose.online/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"test"}'

# Admin panel functionality test
curl -H "Authorization: Bearer $ADMIN_JWT" \
  https://abcdeez.fg-goose.online/api/admin/dashboard
```

---

## 🎯 Specific Incident Scenarios

### **Scenario 1: OAuth Provider Outage**
```
DETECTION: Users cannot sign in with Apple/GitHub
IMPACT: Authentication failures for OAuth users
CONTAINMENT: 
  - Switch to local auth prompts
  - Communicate alternative login methods
  - Monitor provider status pages
RECOVERY: 
  - Wait for provider restoration
  - Test OAuth flows thoroughly
  - Validate user session restoration
```

### **Scenario 2: Admin Panel Compromise**
```
DETECTION: Unauthorized admin panel access detected
IMPACT: Potential full system compromise
CONTAINMENT: 
  - Immediately disable all admin accounts
  - Block suspicious IP addresses
  - Revoke all admin JWT tokens
  - Take admin panel offline
INVESTIGATION:
  - Review audit logs for all admin actions
  - Check for data exfiltration
  - Identify compromise vector
RECOVERY:
  - Reset all admin credentials
  - Implement 2FA requirement
  - Restore admin panel with enhanced monitoring
```

### **Scenario 3: Database Performance Degradation**
```
DETECTION: Slow API responses, high database CPU
IMPACT: Poor user experience, potential timeouts
CONTAINMENT:
  - Enable read-only mode if necessary
  - Scale database resources if possible
  - Implement emergency caching
INVESTIGATION:
  - Identify slow queries
  - Check for unusual traffic patterns
  - Review recent database schema changes
RECOVERY:
  - Optimize problematic queries
  - Add database indexes if needed
  - Implement query monitoring
```

### **Scenario 4: DDoS Attack**
```
DETECTION: Extremely high traffic, rate limiting triggered
IMPACT: Service degradation or outage
CONTAINMENT:
  - Enable aggressive rate limiting
  - Block attack source IPs
  - Activate CDN DDoS protection
INVESTIGATION:
  - Analyze traffic patterns
  - Identify attack vectors
  - Document attack characteristics
RECOVERY:
  - Gradually relax rate limits
  - Implement improved DDoS protection
  - Monitor for sustained attacks
```

---

## 📊 Communication Templates

### **Internal Alert Template**
```
🚨 INCIDENT ALERT - [SEVERITY]

Service: Graph Learning System
Time: [UTC]
Status: [INVESTIGATING/IDENTIFIED/MONITORING/RESOLVED]
Impact: [User/System impact description]
ETA: [Resolution estimate]

Details:
- What: [Brief incident description]
- Impact: [Who/what is affected]
- Actions: [Current response actions]
- Next Update: [When next update will be provided]

Incident Commander: [Name]
War Room: [Slack/Teams/Location]
```

### **User Communication Template**
```
📢 Service Update - [Date/Time]

We're currently experiencing [brief description of issue] affecting [scope of impact].

Current Status: [Working on resolution/Identified issue/Monitoring fix]
Estimated Resolution: [Time estimate or "investigating"]

What you can expect:
- [List of service impacts]
- [Any workarounds available]
- [Timeline for next update]

We apologize for any inconvenience and will provide updates every [frequency].

For real-time updates: [status page link]
```

### **Post-Incident Summary Template**
```
📋 Incident Report - [Incident ID] - [Date]

SUMMARY
Duration: [Start time] to [End time] ([Total duration])
Impact: [Brief impact statement]
Root Cause: [Primary cause]

TIMELINE
[Chronological list of key events]

RESOLUTION
[What fixed the issue]

LESSONS LEARNED
- What went well
- What could be improved
- Action items with owners and due dates

PREVENTION
[Future prevention measures]
```

---

## 🛠️ Tools & Resources

### **Monitoring & Diagnostics**
```bash
# System Health Dashboard
https://abcdeez.fg-goose.online/admin

# API Health Checks
curl https://abcdeez.fg-goose.online/health/health

# Performance Metrics
curl https://abcdeez.fg-goose.online/health/performance

# Database Status
curl https://abcdeez.fg-goose.online/health/ready
```

### **Administrative Tools**
```bash
# Admin Panel
https://abcdeez.fg-goose.online/admin

# GitHub Actions
https://github.com/your-org/graph-learning-system/actions

# Deployment History
gh run list --workflow=deploy-production.yml

# Manual Deployment
gh workflow run deploy-production.yml
```

### **Emergency Contacts**
```
Primary On-Call: [Name] - [Phone] - [Slack]
Secondary: [Name] - [Phone] - [Slack]
Engineering Lead: [Name] - [Email] - [Phone]
Product Owner: [Name] - [Email]
Security Team: [security-team@company.com]
```

### **External Dependencies**
```
Apple Developer Status: https://developer.apple.com/system-status/
GitHub Status: https://www.githubstatus.com/
Your Hosting Provider: [Status page URL]
Database Provider: [Status page URL]
```

---

## 📚 Post-Incident Activities

### **Immediate Post-Incident (Within 24 hours)**
1. **Service Validation**: Ensure all functionality is fully restored
2. **Monitoring Enhancement**: Add new alerts based on incident learnings
3. **Quick Debrief**: Initial lessons learned discussion with response team
4. **User Communication**: Final "all clear" message to users

### **Short-term (Within 1 week)**
1. **Detailed Post-Mortem**: Comprehensive incident analysis meeting
2. **Action Item Tracking**: Assign and track improvement tasks
3. **Documentation Updates**: Update procedures based on learnings
4. **Training Needs Assessment**: Identify team training opportunities

### **Long-term (Within 1 month)**
1. **Process Improvements**: Implement procedural changes
2. **Technical Improvements**: Complete identified technical debt
3. **Disaster Recovery Testing**: Test and improve backup/recovery procedures
4. **Security Assessment**: Comprehensive security review if applicable

---

## 🔄 Testing & Drills

### **Monthly Drills**
- **Failover Testing**: Test switching between environments
- **Backup Restoration**: Verify database backup integrity
- **Communication Testing**: Practice incident communication procedures
- **Tool Verification**: Ensure all monitoring and response tools work

### **Quarterly Assessments**
- **Incident Response Table-Top Exercises**: Simulate major incidents
- **Security Penetration Testing**: Professional security assessment
- **Business Continuity Planning**: Full disaster recovery testing
- **Process Review**: Update procedures based on lessons learned

---

## ⚡ Quick Reference Commands

### **Emergency Admin Actions**
```bash
# Disable user account
curl -X PUT https://abcdeez.fg-goose.online/api/admin/users/{id} \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -d '{"status": "disabled"}'

# Enable maintenance mode
curl -X PUT https://abcdeez.fg-goose.online/api/admin/config \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -d '{"maintenance_mode": "true"}'

# Emergency OAuth validation
curl -X POST https://abcdeez.fg-goose.online/api/admin/oauth-validation \
  -H "Authorization: Bearer $ADMIN_JWT"

# Force background job
curl -X POST https://abcdeez.fg-goose.online/api/admin/jobs \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -d '{"job_type": "emergency_cleanup"}'
```

### **System Status Verification**
```bash
# Full health check
curl -s https://abcdeez.fg-goose.online/health/health | jq '.'

# Database connectivity
curl -s https://abcdeez.fg-goose.online/health/ready

# Performance baseline
curl -s https://abcdeez.fg-goose.online/health/performance | jq '.response_time_ms'

# Recent activity
curl -H "Authorization: Bearer $ADMIN_JWT" \
  "https://abcdeez.fg-goose.online/api/admin/audit?limit=10"
```

---

**🔒 This document is confidential and should only be accessible to authorized incident response team members. Store in a secure, easily accessible location for emergency situations.**

---

*Last Updated: $(date)*
*Version: 1.0*
*Owner: Security & Operations Team*