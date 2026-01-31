# LoggyXP Architecture

![Alt text](screenshots/LoggyXP_Architecture_Diagram.svg)

## Component Overview

### dashboard

* Shows log information in real time and offline mode.
* Control panel for filtering, searching, monitoring, and notifications.
* Allows users to add log files, set filters, notifications, and perform searches (string/regex).
* JSON logs are displayed in a pretty-printed format; only search is available for JSON.

### rust_server

* HTTP and WebSocket API for communication between LogMgr and Web UI.
* Handles client connections and routes commands (add/remove logs, search, filter, notify).
* Broadcasts log updates and notifications to connected clients in real time.

### log_mgr

* Central application manager.
* Coordinates LogFiltering, SearchEngine, LogVisualizer, LogMonitoring, and Notification components.
* Manages shared state for filters and notifications.
* Handles commands from the WebServer and updates log watchers accordingly.

### log_context_data

* Receives user-defined and custom patterns to filter data in one or more logs.
* Real-time tail filtering for line-based logs only.
* Allows users to set alerts for specific patterns in line-based logs.
* Prioritizes and sends notifications to the WebUI when patterns are matched.
* Not available for JSON logs.

### search_engine

* Handles searching through logs using string or regex patterns.
* Supports both line-based and JSON logs (search is the only feature available for JSON).


### log_monitoring

* Watches files for changes using async IO.
* Handles multiple logs from the system storage.
* Supports tailing, filtering, and notifications for line-based logs only.
* For JSON logs, only initial display and search are supported.
* Formats log output for the WebUI.
* Combines results from other components for visualization.
* Pretty-prints JSON logs for display.


---

## Log File Support

- **Line-based logs (e.g., `.txt`, `.log`):**
  - Full support for tailing, filtering, and notifications.
- **JSON logs:**
  - Only display (pretty-printed) and search are supported.

---

## Data Flow

1. **User adds a log file via the WebUI.**
2. **WebServer** receives the request and instructs **LogMgr** to watch the file.
3. **LogMonitoring** starts watching the file for changes.
4. For line-based logs:
    - **LogFiltering** and **Notification** components process new lines in real time.
    - **LogVisualizer** formats and sends updates to the WebUI.
5. For JSON logs:
    - Only initial display and search are available.
    - No tailing, filtering, or notifications.
6. **SearchEngine** processes search requests (string or regex) for both log types.
7. **WebUI** displays results and notifications to the user.

---

## Extending & Customizing

- Add new log types or processing logic by extending the relevant components.
- UI can be customized in `static/dashboard.html`.
- Architecture supports multiple concurrent log watchers and clients.

---