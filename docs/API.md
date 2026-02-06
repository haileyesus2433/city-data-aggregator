# API Documentation

## Overview

This document describes the API contracts between the microservices in the City Data Aggregator system.

## Service Endpoints

### Auth Service (Port 3001)

#### Authentication Endpoints

**POST /api/auth/login**
- Description: Authenticate a user and receive a JWT token
- Request Body:
  ```json
  {
    "username": "string",
    "password": "string"
  }
  ```
- Response: 200 OK
  ```json
  {
    "token": "string",
    "user": {
      "id": "uuid",
      "username": "string",
      "email": "string",
      "role": "string",
      "created_at": "ISO8601"
    }
  }
  ```

**POST /api/auth/register**
- Description: Register a new user
- Request Body:
  ```json
  {
    "username": "string",
    "email": "string",
    "password": "string",
    "role": "string (optional, default: 'user')"
  }
  ```
- Response: 200 OK
  ```json
  {
    "id": "uuid",
    "username": "string",
    "email": "string",
    "role": "string",
    "created_at": "ISO8601"
  }
  ```

#### Admin Endpoints (Require JWT with admin role)

**GET /api/admin/users**
- Description: List all users
- Headers: `Authorization: Bearer <token>`
- Response: 200 OK
  ```json
  [
    {
      "id": "uuid",
      "username": "string",
      "email": "string",
      "role": "string",
      "created_at": "ISO8601"
    }
  ]
  ```

**POST /api/admin/users**
- Description: Create a new user (admin only)
- Headers: `Authorization: Bearer <token>`
- Request Body: Same as register
- Response: 200 OK (UserResponse)

**GET /api/admin/users/{id}**
- Description: Get user by ID
- Headers: `Authorization: Bearer <token>`
- Response: 200 OK (UserResponse)

**DELETE /api/admin/users/{id}**
- Description: Delete a user
- Headers: `Authorization: Bearer <token>`
- Response: 204 No Content

**PUT /api/admin/users/{id}/role**
- Description: Update user role
- Headers: `Authorization: Bearer <token>`
- Request Body:
  ```json
  {
    "role": "string"
  }
  ```
- Response: 200 OK (UserResponse)

### Weather Service (Port 3002)

**GET /api/weather/{city}**
- Description: Get weather data for a city
- Parameters:
  - `city` (path): City name
- Response: 200 OK
  ```json
  {
    "temperature": 15.5,
    "condition": "Partly cloudy",
    "humidity": 65.0,
    "wind_speed": 10.2
  }
  ```

**GET /api/aggregate?city=London&city=Tokyo&city=New+York**
- Description: Aggregate weather and time data for multiple cities (1-20)
- Parameters:
  - `city` (query, repeated): City names (1-20 cities)
- Response: 200 OK
  ```json
  {
    "cities": [
      {
        "city": "London",
        "weather": {
          "temperature": 15.5,
          "condition": "Partly cloudy",
          "humidity": 65.0,
          "wind_speed": 10.2
        },
        "time": {
          "datetime": "2024-01-01T12:00:00+00:00",
          "timezone": "Europe/London",
          "unix_time": 1704110400
        },
        "errors": []
      }
    ],
    "summary": {
      "total": 3,
      "successful": 2,
      "failed": 1
    }
  }
  ```

### Time Service (Port 3003)

**GET /api/time/{city}**
- Description: Get current time for a city
- Parameters:
  - `city` (path): City name
- Response: 200 OK
  ```json
  {
    "datetime": "2024-01-01T12:00:00+00:00",
    "timezone": "Europe/London",
    "unix_time": 1704110400
  }
  ```

## Error Responses

All endpoints may return the following error responses:

- **400 Bad Request**: Invalid request parameters
- **401 Unauthorized**: Missing or invalid JWT token
- **403 Forbidden**: Insufficient permissions
- **404 Not Found**: Resource not found
- **500 Internal Server Error**: Server error

Error response format:
```json
{
  "error": "Error message"
}
```

## Authentication

Most endpoints require JWT authentication. Include the token in the Authorization header:

```
Authorization: Bearer <token>
```

Tokens are obtained via the `/api/auth/login` endpoint and expire after 24 hours.

## Concurrency Model

The `/api/aggregate` endpoint implements the following concurrency model:

- **Maximum 10 in-flight city tasks**: Additional cities queue and wait for permits
- **Parallel fetching**: Weather and time APIs are called in parallel for each city
- **2-second timeout**: Each external API call has a 2-second timeout
- **Exponential backoff retry**: 2 retry attempts with exponential backoff (100ms, 200ms)

## Rate Limiting

- Weather service: 60 requests per minute per client (configurable)
- Time service: No rate limiting (uses cached data)
- Aggregate endpoint: Maximum 10 concurrent city tasks via semaphore

## Timeouts

- External API calls: 2 seconds
- Retry attempts: 2 with exponential backoff

## Graceful Shutdown

All services support graceful shutdown on SIGINT/SIGTERM:

1. Stop accepting new HTTP requests
2. Cancel all in-flight aggregate requests via CancellationToken
3. Wait for existing requests to complete
4. Flush tracing logs before exit

