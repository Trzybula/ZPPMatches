#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}TEST SETUP SCRIPT${NC}"
echo -e "${YELLOW}Backend must be running on http://localhost:3000/${NC}"
echo

make_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4
    
    echo -e "${BLUE}${description}${NC}"
    if [ -n "$data" ]; then
        response=$(curl -s -X $method "http://localhost:3000$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    else
        response=$(curl -s -X $method "http://localhost:3000$endpoint")
    fi
    
    echo "$response"
    echo
}

echo -e "${YELLOW}Cleaning up...${NC}"
rm -f backend/state.json 2>/dev/null
echo "✅ Old state removed"
echo

# 1. REGISTER GROUPS
echo -e "${GREEN}REGISTERING GROUPS${NC}"

make_request POST "/group" \
    '{"name":"g1","email":"g1@test.com","password":"test","preferences":["c1","c2","c3"]}' \
    "Registering group g1"

make_request POST "/group" \
    '{"name":"g2","email":"g2@test.com","password":"test","preferences":["c2","c1","c3"]}' \
    "Registering group g2"

make_request POST "/group" \
    '{"name":"g3","email":"g3@test.com","password":"test","preferences":["c3","c2","c1"]}' \
    "Registering group g3"

make_request POST "/group" \
    '{"name":"g4","email":"g4@test.com","password":"test","preferences":["c1","c2"]}' \
    "Registering group g4"

make_request POST "/group" \
    '{"name":"g5","email":"g5@test.com","password":"test","preferences":["c3"]}' \
    "Registering group g5 (only 1 preference)"

make_request POST "/group" \
    '{"name":"g6","email":"g6@test.com","password":"test","preferences":[]}' \
    "Registering group g6 (no preferences)"

echo

# 2. REGISTER COMPANIES
echo -e "${GREEN}REGISTERING COMPANIES${NC}"

make_request POST "/company" \
    '{"name":"c1","email":"c1@test.com","password":"test","preferences":["g1","g2","g3","g4"]}' \
    "Registering company c1"

make_request POST "/company" \
    '{"name":"c2","email":"c2@test.com","password":"test","preferences":["g2","g3","g1"]}' \
    "Registering company c2"

make_request POST "/company" \
    '{"name":"c3","email":"c3@test.com","password":"test","preferences":["g3","g1","g2"]}' \
    "Registering company c3"

make_request POST "/company" \
    '{"name":"c4","email":"c4@test.com","password":"test","preferences":["g4","g1"]}' \
    "Registering company c4"

make_request POST "/company" \
    '{"name":"c5","email":"c5@test.com","password":"test","preferences":["g1","g2"]}' \
    "Registering company c5"

echo

# 3. LOGIN AND ADD MORE PREFERENCES
echo -e "${GREEN}ADDING EXTRA PREFERENCES${NC}"

echo -e "${BLUE}Logging in as g1 to add more preferences...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST "http://localhost:3000/login/group" \
    -H "Content-Type: application/json" \
    -d '{"email":"g1@test.com","password":"test"}')

SESSION_G1=$(echo $LOGIN_RESPONSE | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
echo "Session ID for g1: $SESSION_G1"

make_request POST "/group/add_pref" \
    "{\"session_id\":\"$SESSION_G1\",\"pref\":\"c4\"}" \
    "Adding c4 to g1's preferences"

make_request POST "/group/add_pref" \
    "{\"session_id\":\"$SESSION_G1\",\"pref\":\"c5\"}" \
    "Adding c5 to g1's preferences"

echo

# 4. CHECK CURRENT DATA
echo -e "${GREEN}CHECKING CURRENT DATA${NC}"

make_request GET "/group/list" "" "All groups"
make_request GET "/company/list" "" "All companies"

echo -e "${BLUE}Checking g1's current preferences:${NC}"
curl -s "http://localhost:3000/group/me?session_id=$SESSION_G1" | python3 -m json.tool
echo

# 5. RUN MATCHING
echo -e "${GREEN}RUNNING MATCHING ALGORITHM ${NC}"
echo -e "${YELLOW}Note: Check backend console for detailed matching logs${NC}"
echo

make_request GET "/match" "" "Running matching algorithm"

echo

# 6. CREATE SOME UNMATCHABLE CASES (FOR TESTING)
echo -e "${GREEN}CREATING SPECIAL TEST CASES${NC}"

make_request POST "/group" \
    '{"name":"g7","email":"g7@test.com","password":"test","preferences":["nonexistent1","nonexistent2"]}' \
    "Registering group g7 (non-existent company preferences)"

make_request POST "/company" \
    '{"name":"c6","email":"c6@test.com","password":"test","preferences":["nonexistent_group1","nonexistent_group2"]}' \
    "Registering company c6 (non-existent group preferences)"

echo
