# Webrana API Gateway Specification

## Overview

Webrana API Gateway provides built-in LLM access for Webrana CLI users without requiring their own API keys.

**Domain**: api.webranaai.com
**Target Launch**: Week 2 December 2024

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    USER DEVICE                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │              WEBRANA CLI                         │   │
│  │  - Device token (auto-generated)                │   │
│  │  - Local usage cache                            │   │
│  │  - Fallback to user API key                     │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│              WEBRANA API GATEWAY                        │
│              api.webranaai.com                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Rust (Axum) + SQLite + Redis                   │   │
│  │  - Device authentication                        │   │
│  │  - Usage tracking & limits                      │   │
│  │  - Rate limiting (token bucket)                 │   │
│  │  - Model routing & load balancing               │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
┌──────────────────────┐   ┌──────────────────────┐
│        GROQ          │   │       GEMINI         │
│  (Primary - Fast)    │   │  (Fallback)          │
│  llama-3.3-70b       │   │  gemini-2.0-flash    │
│  6000 req/day free   │   │  1500 req/day free   │
└──────────────────────┘   └──────────────────────┘
```

## Infrastructure

### VPS Specification

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Provider | Vultr Singapore | Low latency for SEA users |
| Spec | 2 vCPU, 4GB RAM, 80GB SSD | Sufficient for initial launch |
| Cost | $24/month | Reliable, good support |
| OS | Ubuntu 24.04 LTS | Stable, well-supported |

### Tech Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Framework | Rust (Axum) | High performance, low memory |
| Database | SQLite | Simple, no extra service |
| Cache | Redis | Rate limiting, sessions |
| Reverse Proxy | Caddy | Auto HTTPS, simple config |
| Container | Docker + Compose | Easy deployment |
| Monitoring | Prometheus + Grafana | Optional, for metrics |

## API Endpoints

### Authentication

#### POST /v1/auth/register
Register a new device.

**Request:**
```json
{
  "device_id": "sha256-fingerprint",
  "device_name": "MacBook Pro",
  "os": "darwin",
  "cli_version": "0.5.0"
}
```

**Response:**
```json
{
  "token": "wbr_xxxxxxxxxxxxx",
  "tier": "free",
  "limits": {
    "requests_per_day": 50,
    "tokens_per_day": 100000
  },
  "expires_at": null
}
```

#### GET /v1/auth/status
Check current usage and limits.

**Headers:**
```
Authorization: Bearer wbr_xxxxxxxxxxxxx
```

**Response:**
```json
{
  "tier": "free",
  "usage": {
    "requests_today": 23,
    "tokens_today": 45000,
    "requests_limit": 50,
    "tokens_limit": 100000
  },
  "resets_at": "2024-12-09T00:00:00Z"
}
```

### Chat Completions

#### POST /v1/chat/completions
Send chat request (OpenAI-compatible format).

**Headers:**
```
Authorization: Bearer wbr_xxxxxxxxxxxxx
Content-Type: application/json
```

**Request:**
```json
{
  "model": "webrana-default",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "stream": true,
  "max_tokens": 4096
}
```

**Response (streaming):**
```
data: {"choices":[{"delta":{"content":"Hello"}}]}
data: {"choices":[{"delta":{"content":"!"}}]}
data: [DONE]
```

## Pricing Tiers

| Tier | Price | Requests/Day | Tokens/Day | Features |
|------|-------|--------------|------------|----------|
| Free | $0 | 50 | 100K | Basic model, core features |
| Pro | $10/mo | 500 | 1M | Priority routing, all models |
| Team | $25/user/mo | 2000 | 5M | Team management, analytics |
| Enterprise | Custom | Unlimited | Unlimited | Self-hosted, SLA, support |

## Rate Limiting

### Algorithm: Token Bucket

```
Bucket capacity: tier.requests_per_day
Refill rate: capacity / 86400 (per second)
```

### Headers

```
X-RateLimit-Limit: 50
X-RateLimit-Remaining: 47
X-RateLimit-Reset: 1702080000
```

### Error Response (429)

```json
{
  "error": {
    "code": "rate_limit_exceeded",
    "message": "Daily limit reached. Resets at 2024-12-09T00:00:00Z",
    "type": "rate_limit_error"
  }
}
```

## Model Routing

```
Request
   │
   ▼
┌─────────────────┐
│ Check user tier │
└─────────────────┘
   │
   ├─── Free ────▶ Groq (llama-3.3-70b)
   │                  │
   │                  ├─── Success ──▶ Return
   │                  │
   │                  └─── Fail ─────▶ Gemini (fallback)
   │
   └─── Pro ─────▶ Best available model
                     (Claude/GPT-4 if budget allows)
```

## Security

### Device Authentication

1. First run: CLI generates device fingerprint (hardware + random)
2. CLI calls /v1/auth/register with fingerprint
3. Server returns JWT token
4. Token stored in `~/.config/webrana/credentials.json`
5. Subsequent requests include token in Authorization header

### Token Format

```
wbr_[base64-encoded-jwt]

JWT Payload:
{
  "sub": "device_id",
  "tier": "free",
  "iat": 1702000000,
  "exp": null  // Never expires for free tier
}
```

### Security Measures

- All traffic over HTTPS (Caddy auto-cert)
- Rate limiting per device
- Request logging for abuse detection
- IP-based fallback limiting
- Token revocation capability

## CLI Integration

### New Commands

```bash
# Auto-login (happens on first request)
webrana chat "Hello"
# Output: [*] Registered as free tier (50 req/day)

# Check status
webrana status
# Output:
# Tier: Free
# Usage: 23/50 requests today
# Resets: 2024-12-09 00:00 UTC

# Upgrade
webrana upgrade
# Opens: https://webranaai.com/pricing

# Use own API key (bypass gateway)
webrana --api-key sk-xxx chat "Hello"
```

### Config Priority

1. Command line `--api-key` flag
2. Environment variable `OPENAI_API_KEY` / `ANTHROPIC_API_KEY`
3. Webrana built-in model (default)

## Database Schema

```sql
-- Users/Devices
CREATE TABLE devices (
    id TEXT PRIMARY KEY,
    device_name TEXT,
    os TEXT,
    cli_version TEXT,
    tier TEXT DEFAULT 'free',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_seen_at DATETIME
);

-- Usage tracking
CREATE TABLE usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT REFERENCES devices(id),
    date DATE,
    requests INTEGER DEFAULT 0,
    tokens INTEGER DEFAULT 0,
    UNIQUE(device_id, date)
);

-- Request logs
CREATE TABLE request_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT,
    provider TEXT,
    model TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    latency_ms INTEGER,
    status INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Deployment

### Docker Compose

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=sqlite:///data/webrana.db
      - REDIS_URL=redis://redis:6379
      - GROQ_API_KEY=${GROQ_API_KEY}
      - GEMINI_API_KEY=${GEMINI_API_KEY}
    volumes:
      - ./data:/data
    depends_on:
      - redis

  redis:
    image: redis:alpine
    volumes:
      - redis_data:/data

  caddy:
    image: caddy:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data

volumes:
  redis_data:
  caddy_data:
```

### Caddyfile

```
api.webranaai.com {
    reverse_proxy api:3000
}
```

## Implementation Timeline

| Day | Task | Output |
|-----|------|--------|
| 1 | VPS setup, Docker, Caddy | Infrastructure ready |
| 2 | API skeleton (Axum routes) | Basic endpoints |
| 3 | Groq + Gemini integration | LLM proxy working |
| 4 | Auth + rate limiting | Security layer |
| 5 | CLI integration | webrana status, auto-auth |
| 6 | Testing + fixes | Stable system |
| 7 | Deploy + docs | Production launch |

## Future Enhancements

- [ ] Web dashboard for usage stats
- [ ] Stripe integration for Pro tier
- [ ] Team management
- [ ] Custom fine-tuned models
- [ ] Self-hosted Enterprise option
- [ ] Usage analytics API

---

*Document Version: 1.0*
*Last Updated: December 2024*
*Author: NEXUS (Team Alpha Lead)*
