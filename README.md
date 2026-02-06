# City Data Aggregator

A microservices architecture in Rust that aggregates weather and time data for multiple cities, featuring JWT authentication, RBAC, caching, and structured observability.

## Architecture

The system consists of three microservices:

1. **Auth Service** (Port 3001): JWT-based authentication with RBAC
2. **Weather Service** (Port 3002): Weather data aggregation with caching and rate limiting
3. **Time Service** (Port 3003): Time data with startup cache prefill

## Features

- ✅ JWT authentication with stateless architecture
- ✅ Role-based access control (RBAC)
- ✅ PostgreSQL persistence for users and permissions
- ✅ Weather data aggregation from Open-Meteo API
- ✅ Time data from WorldTimeAPI
- ✅ Caching layer with TTL
- ✅ Rate limiting and debounce
- ✅ Concurrent aggregation (max 10 in-flight tasks)
- ✅ 2-second timeout with 2 retry attempts
- ✅ Graceful shutdown
- ✅ Structured observability with tracing
- ✅ OpenAPI documentation with Swagger UI
- ✅ Docker and docker-compose deployment

## Quick Start

### Prerequisites

- Rust 1.88+
- PostgreSQL 16+
- Docker and Docker Compose (optional)

### Local Development

1. **Set up PostgreSQL:**
   ```bash
   # Start PostgreSQL
   docker run -d --name postgres \
     -e POSTGRES_USER=citydata \
     -e POSTGRES_PASSWORD=citydata123 \
     -e POSTGRES_DB=citydata \
     -p 5432:5432 \
     postgres:16-alpine
   ```

2. **Set environment variables:**
   ```bash
   export DATABASE_URL=postgres://citydata:citydata123@localhost:5432/citydata
   export JWT_SECRET=jwt-secret
   ```

3. **Run services:**
   ```bash
   # Terminal 1: Auth Service
   cd auth-service && cargo run

   # Terminal 2: Weather Service
   cd weather-service && cargo run

   # Terminal 3: Time Service
   cd time-service && cargo run
   ```

## API Documentation

Each service exposes Swagger UI for interactive API documentation:

- Auth Service: http://localhost:3001/swagger-ui
- Weather Service: http://localhost:3002/swagger-ui
- Time Service: http://localhost:3003/swagger-ui

See [docs/API.md](docs/API.md) for detailed API contracts.

## Example Usage

### 1. Register a user
```bash
curl -X POST http://localhost:3001/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "email": "admin@example.com",
    "password": "password123",
    "role": "admin"
  }'
```

### 2. Login
```bash
curl -X POST http://localhost:3001/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password123"
  }'
```

### 3. Get weather for a city
```bash
curl http://localhost:3002/api/weather/London
```

### 4. Aggregate data for multiple cities
```bash
curl "http://localhost:3002/api/aggregate?city=London&city=Tokyo&city=New+York"
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run integration tests
cd weather-service && cargo test --test integration_test
```

## Project Structure

```
city-data-aggregator/
├── common/              # Shared library (errors, models, HTTP client)
├── auth-service/        # Authentication service
├── weather-service/     # Weather aggregation service
├── time-service/        # Time service
├── docs/               # Documentation
│   ├── API.md          # API contracts
│   └── flowchart.md    # Architecture flowcharts
└── docker-compose.yml   # Docker orchestration
```

## Environment Variables

### Auth Service
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT signing
- `PORT`: Service port (default: 3001)

### Weather Service
- `PORT`: Service port (default: 3002)
- `OPEN_METEO_URL`: Open-Meteo API URL
- `TIME_SERVICE_URL`: Time service URL
- `CACHE_TTL_SECONDS`: Cache TTL (default: 300)
- `RATE_LIMIT_PER_MINUTE`: Rate limit (default: 60)

### Time Service
- `PORT`: Service port (default: 3003)
- `WORLD_TIME_API_URL`: WorldTimeAPI URL

