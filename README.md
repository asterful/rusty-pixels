# Pixie - r/Place-like Collaborative Canvas

A WebSocket-based collaborative pixel canvas where multiple clients can paint pixels in real-time.

## Features

- 128x128 pixel canvas
- Real-time WebSocket communication
- Broadcasts updates to all connected clients
- Client count tracking with ping/pong messages
- Full board state sent on connection
- Color validation and parsing

## Running the Server

```bash
cargo run
```

The server will start on `127.0.0.1:8080`.

## Testing

Open `test_client.html` in your browser(s) to test the server:

1. The client automatically connects on page load
2. Click anywhere on the canvas to paint a pixel
3. Select different colors using the color picker
4. Click "Ping" to see the current number of connected clients
5. Open multiple browser windows to see real-time collaboration

## Message Protocol

### Client → Server

**Paint a pixel:**
```json
{
  "type": "paint",
  "x": 64,
  "y": 64,
  "color": "#FF5733"
}
```

**Request client count:**
```json
{
  "type": "ping"
}
```

### Server → Client

**Initial board state (sent on connection):**
```json
{
  "type": "init",
  "width": 128,
  "height": 128,
  "board": [
    ["#FFFFFF", "#FFFFFF", ...],
    ...
  ],
  "cooldown": 0
}
```

**Pixel update (broadcast to all clients):**
```json
{
  "type": "update",
  "x": 64,
  "y": 64,
  "color": "#FF5733"
}
```

**Client count response:**
```json
{
  "type": "pong",
  "clients": 5
}
```

## Architecture

### World Management
- `World` struct holds the canvas and history
- Canvas is 128x128 pixels, initialized to white (#FFFFFF)
- Each pixel change is validated and applied to the canvas

### Connection Handling
- Each WebSocket connection receives the full board state immediately
- Paint messages are validated (coordinates, color format)
- Updates are broadcast to all connected clients (including sender)
- Clients are tracked and counted for ping/pong functionality

### Message Flow
1. Client connects → Server sends `init` with full board
2. Client sends `paint` → Server validates and updates canvas
3. Server broadcasts `update` to all clients
4. Client sends `ping` → Server responds with `pong` containing client count

## Project Structure

```
src/
├── main.rs              # Entry point
├── server/
│   ├── mod.rs           # WebSocket server and connection handling
│   └── messages.rs      # Message type definitions
└── world/
    ├── mod.rs           # World state management
    ├── canvas.rs        # Pixel canvas data structure
    ├── color.rs         # Color parsing and conversion
    ├── history.rs       # Change history tracking
    └── change.rs        # Change event definitions
```
