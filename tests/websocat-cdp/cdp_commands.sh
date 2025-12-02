#!/bin/bash
# CDP Command Library
# Provides variety of Chrome DevTools Protocol commands for testing

# Counter for unique IDs
CDP_ID=1

# Get next unique ID
get_next_id() {
    echo $CDP_ID
    CDP_ID=$((CDP_ID + 1))
}

# Browser commands
cdp_browser_version() {
    local id=$(get_next_id)
    echo "{\"id\":$id,\"method\":\"Browser.getVersion\"}"
}

cdp_browser_targets() {
    local id=$(get_next_id)
    echo "{\"id\":$id,\"method\":\"Target.getTargets\"}"
}

# Page commands
cdp_page_navigate() {
    local id=$(get_next_id)
    local url="$1"
    echo "{\"id\":$id,\"method\":\"Page.navigate\",\"params\":{\"url\":\"$url\"}}"
}

cdp_page_reload() {
    local id=$(get_next_id)
    echo "{\"id\":$id,\"method\":\"Page.reload\"}"
}

# Runtime commands
cdp_runtime_evaluate() {
    local id=$(get_next_id)
    local expr="$1"
    echo "{\"id\":$id,\"method\":\"Runtime.evaluate\",\"params\":{\"expression\":\"$expr\"}}"
}

# DOM commands
cdp_dom_get_document() {
    local id=$(get_next_id)
    echo "{\"id\":$id,\"method\":\"DOM.getDocument\"}"
}

# Network commands
cdp_network_enable() {
    local id=$(get_next_id)
    echo "{\"id\":$id,\"method\":\"Network.enable\"}"
}

cdp_network_disable() {
    local id=$(get_next_id)
    echo "{\"id\":$id,\"method\":\"Network.disable\"}"
}

# Invalid command for error testing
cdp_invalid_method() {
    local id=$(get_next_id)
    echo "{\"id\":$id,\"method\":\"Invalid.NonExistent\"}"
}

cdp_invalid_json() {
    echo "{this is not valid json"
}

# Get random command (excluding invalid)
cdp_random_valid() {
    local commands=(
        "cdp_browser_version"
        "cdp_browser_targets"
        "cdp_runtime_evaluate '1+1'"
        "cdp_runtime_evaluate 'navigator.userAgent'"
    )
    local idx=$((RANDOM % ${#commands[@]}))
    eval "${commands[$idx]}"
}
