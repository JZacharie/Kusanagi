# OpenObserve Alert Rules for Kusanagi

## Alert Configuration

These alert rules should be configured in OpenObserve to monitor Kusanagi dashboard health and performance.

## 1. High Error Rate Alert

**Alert Name**: `kusanagi-high-error-rate`  
**Severity**: `critical`  
**Condition**: Error rate exceeds 5% over 5 minutes

```sql
SELECT 
  count(*) FILTER (WHERE severity IN ('error', 'critical')) * 100.0 / count(*) as error_rate
FROM logs 
WHERE service='kusanagi'
  AND timestamp >= now() - interval '5 minutes'
HAVING error_rate > 5
```

**Alert Actions**:
- Send notification to Slack/Email
- Create incident ticket
- Trigger on-call escalation if > 10%

---

## 2. API Performance Degradation

**Alert Name**: `kusanagi-slow-api-calls`  
**Severity**: `warning`  
**Condition**: Average API response time > 2000ms over 5 minutes

```sql
SELECT 
  endpoint,
  avg(duration_ms) as avg_duration
FROM rum 
WHERE service='kusanagi'
  AND action.type='api_call'
  AND timestamp >= now() - interval '5 minutes'
GROUP BY endpoint
HAVING avg_duration > 2000
```

**Alert Actions**:
- Send notification with affected endpoints
- Log to performance monitoring channel

---

## 3. API Failure Spike

**Alert Name**: `kusanagi-api-failures`  
**Severity**: `error`  
**Condition**: API failure rate > 10% over 5 minutes

```sql
SELECT 
  endpoint,
  count(*) FILTER (WHERE success=false) * 100.0 / count(*) as failure_rate
FROM rum 
WHERE service='kusanagi'
  AND action.type='api_call'
  AND timestamp >= now() - interval '5 minutes'
GROUP BY endpoint
HAVING failure_rate > 10
```

**Alert Actions**:
- Send notification with affected API endpoints
- Create incident if failure rate > 25%

---

## 4. Critical JavaScript Errors

**Alert Name**: `kusanagi-critical-errors`  
**Severity**: `critical`  
**Condition**: Any critical error occurs

```sql
SELECT 
  message,
  category,
  filename,
  count(*) as occurrences
FROM logs 
WHERE service='kusanagi'
  AND severity='critical'
  AND timestamp >= now() - interval '1 minute'
GROUP BY message, category, filename
HAVING occurrences > 0
```

**Alert Actions**:
- Immediate notification to development team
- Include error stack trace and context
- Create high-priority incident

---

## 5. Page Load Performance Degradation

**Alert Name**: `kusanagi-slow-page-loads`  
**Severity**: `warning`  
**Condition**: P95 page load time > 5 seconds

```sql
SELECT 
  percentile_cont(0.95) WITHIN GROUP (ORDER BY view.time_spent) as p95_load_time
FROM rum 
WHERE service='kusanagi'
  AND view.url_path != ''
  AND timestamp >= now() - interval '10 minutes'
HAVING p95_load_time > 5000
```

**Alert Actions**:
- Send notification to performance team
- Log detailed performance metrics

---

## 6. Resource Loading Failures

**Alert Name**: `kusanagi-resource-failures`  
**Severity**: `warning`  
**Condition**: More than 5 resource loading failures in 5 minutes

```sql
SELECT 
  resource_url,
  count(*) as failures
FROM logs 
WHERE service='kusanagi'
  AND category='resource_error'
  AND timestamp >= now() - interval '5 minutes'
GROUP BY resource_url
HAVING failures > 5
```

**Alert Actions**:
- Send notification with affected resources
- Check CDN and asset availability

---

## 7. WebSocket Connection Issues

**Alert Name**: `kusanagi-websocket-failures`  
**Severity**: `error`  
**Condition**: Multiple WebSocket connection failures

```sql
SELECT 
  count(*) as ws_errors
FROM logs 
WHERE service='kusanagi'
  AND category='websocket_error'
  AND timestamp >= now() - interval '5 minutes'
HAVING ws_errors > 3
```

**Alert Actions**:
- Send notification to infrastructure team
- Check WebSocket server health

---

## 8. Anomalous User Activity

**Alert Name**: `kusanagi-low-user-activity`  
**Severity**: `info`  
**Condition**: No active users for 30 minutes (during business hours)

```sql
SELECT 
  count(DISTINCT user.id) as active_users
FROM rum 
WHERE service='kusanagi'
  AND timestamp >= now() - interval '30 minutes'
HAVING active_users = 0
```

**Alert Actions**:
- Send informational notification
- Check if service is reachable

---

## Implementation Steps

### 1. Create Alerts in OpenObserve

Navigate to OpenObserve UI:
```
https://o2-openobserve.p.zacharie.org
```

1. Go to **Alerts** section
2. Click **Create Alert**
3. For each alert above:
   - Copy the SQL query
   - Set the alert name
   - Configure severity level
   - Set evaluation interval (typically 1-5 minutes)
   - Add notification channels

### 2. Configure Notification Channels

Set up notification destinations:
- **Slack**: Create webhook for #kusanagi-alerts channel
- **Email**: Add team email addresses
- **PagerDuty**: For critical alerts requiring on-call response

### 3. Test Alerts

After configuration:
1. Trigger test conditions
2. Verify notifications are received
3. Adjust thresholds as needed based on normal traffic patterns

### 4. Alert Tuning

Monitor alert frequency over first week:
- Reduce false positives by adjusting thresholds
- Add filters for known non-issues
- Consider time-of-day patterns for traffic-based alerts

## Alert Escalation Matrix

| Severity | Response Time | Escalation |
|----------|--------------|------------|
| **Critical** | Immediate | On-call engineer → Team lead → Engineering manager |
| **Error** | < 30 minutes | Assigned engineer → Team lead |
| **Warning** | < 2 hours | Monitoring team → Assigned engineer |
| **Info** | Next business day | Logged for review |

## Monitoring Best Practices

1. **Review alerts weekly**: Adjust thresholds based on actual patterns
2. **Document false positives**: Update alert conditions to reduce noise
3. **Track MTTR**: Monitor mean time to resolution for different alert types
4. **Regular testing**: Simulate alert conditions monthly to verify functionality
5. **Alert fatigue prevention**: Consolidate related alerts, use proper severity levels
