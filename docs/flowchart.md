# System Architecture Flowcharts

## Service Architecture

```mermaid
flowchart TB
    subgraph client [Client Layer]
        CLI[HTTP Client]
    end
    
    subgraph services [Microservices]
        subgraph weather [Weather Service - Port 3002]
            AGG["/aggregate endpoint"]
            WAPI["/api/weather endpoint"]
        end
        AUTH[Auth Service<br/>Port 3001]
        TIME[Time Service<br/>Port 3003]
    end
    
    subgraph external [External APIs]
        OPENMETEO[Open-Meteo API]
        WORLDTIME[WorldTimeAPI]
    end
    
    subgraph storage [Storage Layer]
        PG[(PostgreSQL)]
        WCACHE[(Weather Cache)]
        TCACHE[(Time Cache)]
    end
    
    CLI --> AGG
    CLI --> AUTH
    AGG -->|HTTP Call| TIME
    AGG --> WAPI
    WAPI --> OPENMETEO
    WAPI --> WCACHE
    
    AUTH --> PG
    TIME --> WORLDTIME
    TIME --> TCACHE
```

## Service Communication

The services communicate as follows:

1. **Client to Weather Service**: HTTP requests to `/aggregate` or `/api/weather/{city}`
2. **Client to Auth Service**: HTTP requests for login/register and admin operations
3. **Client to Time Service**: HTTP requests to `/api/time/{city}`
4. **Weather Service to Time Service**: HTTP call from Aggregator to fetch time data
5. **Weather Service to Open-Meteo**: HTTP call to external weather API
6. **Time Service to WorldTimeAPI**: HTTP call to external time API

## Aggregate Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant WeatherService as Weather Service<br/>/aggregate
    participant OpenMeteo as Open-Meteo API
    participant TimeService as Time Service
    participant WorldTimeAPI
    
    Client->>WeatherService: GET /aggregate?city=London&city=Tokyo
    WeatherService->>WeatherService: Validate cities 1-20
    
    par Spawn concurrent tasks per city
        Note over WeatherService: Semaphore limits to 10 concurrent
        
        par Parallel fetch for London
            WeatherService->>OpenMeteo: GET forecast?lat=51.5&lon=-0.1
            OpenMeteo-->>WeatherService: Weather data
        and
            WeatherService->>TimeService: GET /api/time/London
            TimeService->>TimeService: Check cache
            alt Cache miss
                TimeService->>WorldTimeAPI: GET /Europe/London
                WorldTimeAPI-->>TimeService: Time data
                TimeService->>TimeService: Update cache
            end
            TimeService-->>WeatherService: Time response
        end
    end
    
    WeatherService->>WeatherService: Aggregate all results
    WeatherService-->>Client: AggregateResponse with summary
```

## Authentication Flow

```mermaid
sequenceDiagram
    participant Client
    participant AuthService as Auth Service
    participant Database as PostgreSQL
    
    Client->>AuthService: POST /api/auth/login
    AuthService->>Database: Query user by username
    Database-->>AuthService: User data
    AuthService->>AuthService: Verify password bcrypt
    AuthService->>Database: Get role permissions
    Database-->>AuthService: Permissions list
    AuthService->>AuthService: Generate JWT token
    AuthService-->>Client: JWT token + user data
    
    Note over Client: Use token for admin endpoints
    
    Client->>AuthService: GET /api/admin/users<br/>Authorization: Bearer token
    AuthService->>AuthService: auth_middleware validates JWT
    AuthService->>AuthService: require_admin checks role
    AuthService->>Database: Query all users
    Database-->>AuthService: Users list
    AuthService-->>Client: Users data
```

## Caching Strategy

### Weather Cache (TTL-based)
```mermaid
flowchart LR
    A[Weather Request] --> B{Cache Hit?}
    B -->|Yes and not expired| C[Return Cached Data]
    B -->|No or expired| D[Fetch from Open-Meteo]
    D --> E{Success?}
    E -->|Yes| F[Store with TTL]
    E -->|No| G[Return Error]
    F --> H[Return Data]
    G --> I[Client]
    H --> I
    C --> I
```

### Time Cache (Startup prefill)
```mermaid
flowchart TD
    A[Service Startup] --> B[Prefill cache with common cities]
    B --> C[London, Tokyo, Paris, etc.]
    C --> D[Cache ready for requests]
    
    E[Time Request] --> F{Cache Hit?}
    F -->|Yes| G[Return Cached Data]
    F -->|No| H[Fetch from WorldTimeAPI]
    H --> I[Store in Cache]
    I --> J[Return Data]
    G --> J
```

## Concurrency Control

```mermaid
flowchart TD
    A[20 Cities Requested] --> B[Spawn 20 tokio tasks]
    B --> C[Semaphore: 10 permits]
    
    subgraph concurrent [Concurrent Execution]
        D[Task 1: Acquire permit]
        E[Task 2: Acquire permit]
        F[...]
        G[Task 10: Acquire permit]
        H[Task 11-20: Wait for permit]
    end
    
    C --> concurrent
    
    D --> I[Fetch Weather + Time in parallel]
    I --> J[Release permit]
    J --> K[Task 11 acquires permit]
    
    concurrent --> L[All tasks complete]
    L --> M[Return aggregated results]
```

## Graceful Shutdown

```mermaid
sequenceDiagram
    participant OS
    participant Server as Axum Server
    participant Token as CancellationToken
    participant Tasks as In-Flight Tasks
    
    OS->>Server: SIGINT or SIGTERM
    Server->>Server: Stop accepting new connections
    Server->>Token: cancel
    Token->>Tasks: Notify cancellation
    Tasks->>Tasks: Check is_cancelled in select!
    Tasks->>Tasks: Return early with cancelled error
    Server->>Server: Wait for shutdown complete
    Note over Server: Tracing logs flushed automatically
    Server->>OS: Exit 0
```

## Error Handling Flow

```mermaid
flowchart TD
    A[API Call] --> B{Timeout 2s?}
    B -->|Yes| C[TimeoutError]
    B -->|No| D{HTTP Status?}
    D -->|4xx/5xx| E[HttpError]
    D -->|Success| F{Parse JSON?}
    F -->|Fail| G[ParseError]
    F -->|Success| H[Return Data]
    
    D -->|Connection failed| I[NetworkError]
    
    C --> J{Retry count < 2?}
    E --> J
    G --> J
    I --> J
    
    J -->|Yes| K[Exponential backoff]
    K --> L[100ms, 200ms delay]
    L --> A
    
    J -->|No| M[Return error to caller]
    
    Note over M: Per-city errors tracked in response
```

## Data Flow Summary

```mermaid
flowchart LR
    subgraph External
        OM[Open-Meteo API]
        WT[WorldTimeAPI]
    end
    
    subgraph Internal
        WS[Weather Service:3002]
        TS[Time Service:3003]
        AS[Auth Service:3001]
        PG[(PostgreSQL)]
    end
    
    subgraph Client
        C[HTTP Client]
    end
    
    C -->|/aggregate| WS
    C -->|/api/auth/*| AS
    C -->|/api/time/*| TS
    
    WS -->|HTTP| TS
    WS -->|HTTP| OM
    TS -->|HTTP| WT
    AS -->|SQL| PG
```
