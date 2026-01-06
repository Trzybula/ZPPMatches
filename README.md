# ZPPMatches
A Rust-based matching system implementing the Gale-Shapley stable matching algorithm. The system matches student groups with companies based on mutual preferences.

## Architecture
- Backend: Rust + Axum
- Frontend: Yew (WASM) + Trunk
- Algorithm: Gale-Shapley
- Data Storage: JSON file (state.json)

## Setup:

### Prepare mock data:
- cd backend

### Run backend:
- cd backend
- cargo run

### Run frontend:
- cd frontend
- trunk serve

## First Iteration Features:
- Login for companies and groups (for now adding only through CURL),
- Dashboards
- Matching algorithm (for now some companies may have more groups than just one, but
in the second iteration it would be a bijection between companies' projects(TODO) and groups
- Matching dashboard

## Second Iteration Features:
- Unifed login page
- Registration
- Adding available companies' projects with their descriptions
- Projects overview
- Rankings and preferences that users can manage
- Algorithm with preferences and ranking, bijection between companies' projects and groups
- Matching rounds
- Admin panel for managing rounds

## TODO:
- Groups overview
- Companies overview
- Database
- Chat between companies and groups
