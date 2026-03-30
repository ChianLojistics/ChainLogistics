# Digital Twin Platform Documentation

## Overview

The ChainLogistics Digital Twin Platform creates virtual replicas of physical supply chain entities, enabling simulation, optimization, and predictive modeling. This powerful feature allows organizations to test scenarios, predict outcomes, and optimize operations before implementing changes in the real world.

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Architecture](#architecture)
4. [Features](#features)
5. [API Reference](#api-reference)
6. [Use Cases](#use-cases)
7. [Implementation Guide](#implementation-guide)
8. [Best Practices](#best-practices)

## Introduction

### What is a Digital Twin?

A digital twin is a virtual representation of a physical object, process, or system that spans its lifecycle and is updated from real-time data. In supply chain management, digital twins enable:

- Real-time monitoring and visualization
- Predictive analytics and forecasting
- What-if scenario simulation
- Optimization recommendations
- Risk assessment and mitigation

### Benefits

- **Risk Reduction**: Test changes in a virtual environment before real-world implementation
- **Cost Optimization**: Identify inefficiencies and optimization opportunities
- **Predictive Insights**: Forecast issues before they occur
- **Data-Driven Decisions**: Make informed decisions based on simulations
- **Continuous Improvement**: Iterate and optimize operations continuously

## Core Concepts

### 1. Digital Twin Types

The platform supports multiple twin types:

**Product Twin**

- Represents individual products or batches
- Tracks lifecycle from origin to destination
- Monitors quality, location, and condition

**Warehouse Twin**

- Models warehouse operations and inventory
- Optimizes storage and retrieval
- Predicts capacity and throughput

**Vehicle Twin**

- Represents transportation assets
- Optimizes routes and schedules
- Monitors performance and maintenance

**Container Twin**

- Tracks shipping containers
- Monitors environmental conditions
- Optimizes loading and routing

**Facility Twin**

- Models production or distribution facilities
- Optimizes workflows and processes
- Predicts capacity and bottlenecks

### 2. Twin States

Digital twins maintain state information that evolves over time:

```json
{
  "location": {
    "latitude": 40.7128,
    "longitude": -74.006,
    "address": "New York, NY"
  },
  "condition": {
    "temperature": 22.5,
    "humidity": 45,
    "quality_score": 0.95
  },
  "status": "in_transit",
  "metrics": {
    "distance_traveled": 1250,
    "time_elapsed_hours": 18,
    "estimated_arrival": "2024-03-20T14:30:00Z"
  }
}
```

### 3. Simulations

Simulations enable what-if analysis:

**Route Optimization**

- Find optimal paths considering distance, time, cost, and carbon
- Account for traffic, weather, and restrictions
- Compare multiple routing strategies

**Demand Forecasting**

- Predict future demand based on historical data
- Identify trends and seasonality
- Account for external factors

**Inventory Optimization**

- Determine optimal stock levels
- Minimize holding costs while avoiding stockouts
- Balance service levels with costs

**Risk Assessment**

- Identify potential disruptions
- Quantify risk levels
- Recommend mitigation strategies

**Cost Analysis**

- Break down total cost of ownership
- Identify cost reduction opportunities
- Compare alternative scenarios

**Timeline Projection**

- Predict project completion times
- Identify critical path and bottlenecks
- Assess schedule risks

### 4. Predictions

AI/ML-based predictions for key events:

- **Arrival Time**: Estimated time of arrival with confidence intervals
- **Delays**: Probability and magnitude of delays
- **Quality Issues**: Risk of quality degradation
- **Demand Spikes**: Unexpected demand increases
- **Supply Disruptions**: Potential supply chain interruptions
- **Cost Overruns**: Budget variance predictions

### 5. Optimizations

Actionable recommendations for improvement:

- Route optimization suggestions
- Inventory level adjustments
- Cost reduction opportunities
- Time savings strategies
- Carbon footprint reduction
- Risk mitigation actions

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Twin Viewer  │  │  Simulation  │  │   Analytics      │  │
│  │  Dashboard   │  │   Console    │  │   Dashboard      │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    API Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Twin API     │  │ Simulation   │  │  Prediction      │  │
│  │              │  │ API          │  │  API             │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Digital Twin Service                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ State Mgmt   │  │ Simulation   │  │  ML Engine       │  │
│  │              │  │ Engine       │  │                  │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Data Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Twin States  │  │ Simulations  │  │  Predictions     │  │
│  │ (PostgreSQL) │  │ (PostgreSQL) │  │  (PostgreSQL)    │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Real-time Updates**: Physical events update twin state
2. **State Synchronization**: Twin state persisted to database
3. **Analytics Processing**: Metrics calculated from state data
4. **Prediction Generation**: ML models generate predictions
5. **Simulation Execution**: What-if scenarios run on demand
6. **Optimization Recommendations**: System suggests improvements

## Features

### 1. Twin Management

**Create Digital Twin**

```bash
POST /api/digital-twins
{
  "product_id": "PROD-12345",
  "twin_type": "Product",
  "name": "Coffee Batch #42 Twin",
  "description": "Digital twin for Ethiopian coffee batch",
  "initial_state": {
    "location": "Addis Ababa, Ethiopia",
    "status": "harvested",
    "quality_score": 0.95
  },
  "metadata": {
    "batch_size": 1000,
    "variety": "Arabica"
  }
}
```

**Update Twin State**

```bash
PUT /api/digital-twins/{twin_id}/state
{
  "state_data": {
    "location": "Port of Djibouti",
    "status": "in_transit",
    "temperature": 22.5
  },
  "metrics": {
    "distance_traveled": 350,
    "time_elapsed_hours": 8
  },
  "source": "iot_sensor"
}
```

### 2. Simulations

**Create Simulation**

```bash
POST /api/simulations
{
  "twin_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Route Optimization Test",
  "description": "Compare shipping routes",
  "simulation_type": "RouteOptimization",
  "parameters": {
    "origin": "Shanghai",
    "destination": "Los Angeles",
    "constraints": {
      "max_cost": 5000,
      "max_time_days": 30
    }
  },
  "created_by": "user@example.com"
}
```

**Run Simulation**

```bash
POST /api/simulations/{simulation_id}/run
```

**Simulation Result**

```json
{
  "simulation_id": "660e8400-e29b-41d4-a716-446655440000",
  "status": "Completed",
  "execution_time_ms": 1250,
  "results": {
    "original_route": {
      "distance_km": 11500,
      "time_days": 28,
      "cost_usd": 4800,
      "carbon_kg": 2300
    },
    "optimized_route": {
      "distance_km": 10800,
      "time_days": 25,
      "cost_usd": 4200,
      "carbon_kg": 2050
    },
    "improvements": {
      "distance_reduction_percent": 6.1,
      "time_savings_days": 3,
      "cost_savings_usd": 600,
      "carbon_reduction_kg": 250
    }
  },
  "insights": [
    "Route optimization can reduce distance by 6.1%",
    "Estimated time savings of 3 days",
    "Carbon emissions reduced by 250kg"
  ],
  "recommendations": [
    "Implement the optimized route",
    "Consider rail transport for inland segments",
    "Monitor weather conditions along route"
  ]
}
```

### 3. Predictions

**Create Prediction**

```bash
POST /api/predictions
{
  "twin_id": "550e8400-e29b-41d4-a716-446655440000",
  "prediction_type": "ArrivalTime",
  "prediction_horizon": 72,
  "input_features": {
    "current_location": "Pacific Ocean",
    "current_speed": 22,
    "weather_conditions": "favorable"
  }
}
```

**Prediction Response**

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440000",
  "twin_id": "550e8400-e29b-41d4-a716-446655440000",
  "prediction_type": "ArrivalTime",
  "predicted_value": {
    "estimated_arrival": "2024-03-20T14:30:00Z",
    "variance_hours": 2
  },
  "confidence_score": 0.85,
  "prediction_horizon": 72,
  "created_at": "2024-03-17T14:30:00Z",
  "valid_until": "2024-03-20T14:30:00Z"
}
```

### 4. Analytics

**Get Twin Analytics**

```bash
GET /api/digital-twins/{twin_id}/analytics
```

**Analytics Response**

```json
{
  "twin_id": "550e8400-e29b-41d4-a716-446655440000",
  "metrics": {
    "total_events": 150,
    "avg_transit_time": 72.5,
    "on_time_delivery_rate": 0.92,
    "quality_score": 0.95,
    "cost_efficiency": 0.88,
    "carbon_footprint": 450.0,
    "risk_score": 0.35
  },
  "recent_predictions": [...],
  "active_optimizations": [...],
  "simulation_count": 12,
  "state_history_count": 150
}
```

## API Reference

### Endpoints

#### Digital Twins

| Method | Endpoint                            | Description           |
| ------ | ----------------------------------- | --------------------- |
| POST   | `/api/digital-twins`                | Create digital twin   |
| GET    | `/api/digital-twins/{id}`           | Get twin by ID        |
| GET    | `/api/products/{id}/twins`          | Get twins for product |
| PUT    | `/api/digital-twins/{id}/state`     | Update twin state     |
| GET    | `/api/digital-twins/{id}/states`    | Get state history     |
| GET    | `/api/digital-twins/{id}/analytics` | Get twin analytics    |

#### Simulations

| Method | Endpoint                              | Description           |
| ------ | ------------------------------------- | --------------------- |
| POST   | `/api/simulations`                    | Create simulation     |
| GET    | `/api/simulations/{id}`               | Get simulation        |
| POST   | `/api/simulations/{id}/run`           | Run simulation        |
| GET    | `/api/digital-twins/{id}/simulations` | List twin simulations |

#### Predictions

| Method | Endpoint                              | Description           |
| ------ | ------------------------------------- | --------------------- |
| POST   | `/api/predictions`                    | Create prediction     |
| GET    | `/api/predictions/{id}`               | Get prediction        |
| GET    | `/api/digital-twins/{id}/predictions` | List twin predictions |

## Use Cases

### 1. Supply Chain Optimization

**Scenario**: A coffee importer wants to optimize shipping routes from Ethiopia to the US.

**Implementation**:

1. Create product twin for coffee batch
2. Run route optimization simulation
3. Compare multiple shipping options
4. Implement optimal route
5. Monitor actual performance vs. prediction

**Benefits**:

- 10-15% cost reduction
- 2-3 days faster delivery
- 20% lower carbon footprint

### 2. Demand Forecasting

**Scenario**: A retailer needs to predict demand for seasonal products.

**Implementation**:

1. Create warehouse twin
2. Run demand forecasting simulation
3. Adjust inventory levels based on predictions
4. Monitor actual demand vs. forecast
5. Refine model with actual data

**Benefits**:

- 25% reduction in stockouts
- 15% lower inventory holding costs
- Improved customer satisfaction

### 3. Risk Management

**Scenario**: A manufacturer wants to assess supply chain risks.

**Implementation**:

1. Create facility twin
2. Run risk assessment simulation
3. Identify high-risk scenarios
4. Implement mitigation strategies
5. Monitor risk indicators

**Benefits**:

- Early warning of disruptions
- 40% faster response to issues
- Reduced impact of disruptions

### 4. Quality Assurance

**Scenario**: A pharmaceutical company needs to ensure product quality during transport.

**Implementation**:

1. Create container twin with sensors
2. Monitor temperature and humidity
3. Predict quality issues
4. Alert on threshold violations
5. Take corrective action

**Benefits**:

- 99.9% quality compliance
- Reduced product waste
- Regulatory compliance

## Implementation Guide

### Step 1: Create Digital Twin

```typescript
// Frontend example
const createTwin = async (productId: string) => {
  const response = await fetch("/api/digital-twins", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      product_id: productId,
      twin_type: "Product",
      name: "My Product Twin",
      description: "Digital twin for tracking",
      initial_state: {
        location: "Origin",
        status: "created",
      },
      metadata: {},
    }),
  });

  return await response.json();
};
```

### Step 2: Update State

```typescript
const updateTwinState = async (twinId: string, stateData: any) => {
  const response = await fetch(`/api/digital-twins/${twinId}/state`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      state_data: stateData,
      metrics: calculateMetrics(stateData),
      source: "manual_update",
    }),
  });

  return await response.json();
};
```

### Step 3: Run Simulation

```typescript
const runSimulation = async (twinId: string) => {
  // Create simulation
  const simulation = await fetch("/api/simulations", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      twin_id: twinId,
      name: "Route Optimization",
      simulation_type: "RouteOptimization",
      parameters: {
        origin: "New York",
        destination: "Los Angeles",
      },
      created_by: "user@example.com",
    }),
  }).then((r) => r.json());

  // Run simulation
  const result = await fetch(`/api/simulations/${simulation.id}/run`, {
    method: "POST",
  }).then((r) => r.json());

  return result;
};
```

### Step 4: Get Analytics

```typescript
const getTwinAnalytics = async (twinId: string) => {
  const response = await fetch(`/api/digital-twins/${twinId}/analytics`);
  return await response.json();
};
```

## Best Practices

### 1. State Management

- Update twin state regularly (every event or time interval)
- Include relevant metrics with each state update
- Use consistent data formats
- Validate state data before updates

### 2. Simulation Design

- Define clear objectives for each simulation
- Use realistic parameters and constraints
- Run multiple scenarios for comparison
- Document assumptions and limitations

### 3. Prediction Accuracy

- Provide sufficient historical data
- Update models with actual outcomes
- Monitor prediction accuracy over time
- Adjust confidence thresholds appropriately

### 4. Performance Optimization

- Limit state history retention (e.g., last 1000 states)
- Use pagination for large result sets
- Cache frequently accessed analytics
- Run simulations asynchronously

### 5. Security

- Authenticate all API requests
- Authorize access to sensitive twins
- Encrypt sensitive state data
- Audit simulation and prediction access

## Advanced Topics

### Custom Simulation Types

Extend the platform with custom simulation logic:

```rust
// Add custom simulation type
pub enum SimulationType {
    // ... existing types
    CustomOptimization,
}

// Implement simulation logic
async fn simulate_custom_optimization(
    &self,
    parameters: &serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    // Your custom logic here
    Ok(serde_json::json!({
        "result": "custom optimization result"
    }))
}
```

### Machine Learning Integration

Integrate ML models for predictions:

```python
# Train prediction model
from sklearn.ensemble import RandomForestRegressor

model = RandomForestRegressor()
model.fit(X_train, y_train)

# Save model
import joblib
joblib.dump(model, 'arrival_time_model.pkl')

# Use in prediction service
model = joblib.load('arrival_time_model.pkl')
prediction = model.predict(features)
```

### Real-time State Updates

Integrate IoT sensors for real-time updates:

```typescript
// WebSocket connection for real-time updates
const ws = new WebSocket("ws://api.chainlogistics.com/ws");

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  if (update.type === "twin_state_update") {
    updateTwinState(update.twin_id, update.state);
  }
};
```

## Troubleshooting

### Common Issues

**Simulation Timeout**

- Reduce simulation complexity
- Optimize parameters
- Run simulations asynchronously

**Inaccurate Predictions**

- Provide more training data
- Update model with actual outcomes
- Adjust feature engineering

**State Sync Issues**

- Check network connectivity
- Verify authentication
- Review error logs

## Support

For questions or issues:

- Documentation: https://docs.chainlogistics.com
- GitHub Issues: https://github.com/ChainLojistics/ChainLogistics/issues
- Discord: https://discord.gg/chainlogistics
- Email: support@chainlogistics.com

## License

MIT License - See LICENSE file for details
