# ZPPMatches
A Rust-based matching system implementing the Gale-Shapley stable matching algorithm. The system matches student groups with companies based on mutual preferences.

## Architecture
- Backend: Rust + Axum
- Frontend: Yew (WASM) + Trunk
- Algorithm: Gale-Shapley
- Data Storage: JSON file (state.json)

## Setup:
### Run backend:
- cd backend
- cargo run

### Prepare mock data:
- cd backend
- bash setup_test.sh
(for now we keep data in state.json)

### Run frontend:
- cd frontend
- trunk serve

## Features:
- Login for companies and groups (for now adding only through CURL),
- Dashboards
- Matching algorithm (for now some companies may have more groups than just 1, but
in the second iteration it would be bijection between companies' projects(TODO) and groups
- Matching dashboard

## TODO:
- Registration
- Adding available companies' projects
- Groups overview
- Companies and projects overview
- Advanced algorithm with preferences
- Admin panel
- Database
- Chat between companies and groups
- Matching rounds
