# LoggyXP

LoggyXP is a Rust-based log monitoring and search application with a web dashboard and real-time notifications. It allows you to watch log files, tail updates, search for patterns (string or regex), and set filters or notifications for specific log events.

---

## Features

- **Watch log files:** Monitor changes to multiple log files in real time.
- **Web dashboard:** View logs and interact with the app via a browser.
- **Search:** Find log lines by string or regex patterns.
- **Filters:** Only display log lines matching specific patterns (line-based logs only).
- **Notifications:** Get notified when log lines match your criteria (line-based logs only).
- **Batch log sending:** Efficiently sends log lines in batches to the client.

---

## Log File Support

- **Line-based logs (e.g., `.txt`, `.log`):**
  - **Tailing, filtering, and notifications are fully supported.**
  - You can watch, tail, filter, and set notifications for these logs.

- **JSON logs:**
  - **Only search is supported.**
  - JSON logs will be displayed in a pretty-printed format.
  - Tailing, filtering, and notifications are **not** available for JSON files.

---

## Project Structure

```
src/
  main.rs                  # Application entry point
  log_mgr.rs               # Main logic and orchestration
  log_mgr/
    log_context_data.rs    # Filter and notification management
    log_monitoring.rs      # File watching and tailing
    rust_server.rs         # HTTP/WebSocket server
    search_engine.rs       # Search utilities (string/regex)

static/
  dashboard.html           # Main dashboard UI

tests/
  generator.sh             # Script to generate test logs
  logs_for_testing/
    dummy_log1.txt         # Example line-based log file
    json1.json             # Example JSON log file
    Apache_2k.log          # Example Apache log file
    dns_with_timestamps.log # Example DNS log file with timestamps
```

---

## Getting Started

### Prerequisites

- Rust (latest stable)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started.html)

### Build & Run

1. Clone the repository.
2. Build the project:
    ```sh
    cargo build --release
    ```
3. Run the application:
    ```sh
    cargo run --release
    ```
4. Open your browser and go to [http://127.0.0.1:3000](http://127.0.0.1:3000) to access the dashboard.

---

## How to Add Log Files

1. **Start LoggyXP** and open the dashboard in your browser.
2. **Locate the "Add Log File" or "Watch Log" input** on the dashboard (usually a text field and button).
   - The main dashboard is `static/dashboard.html`.
3. **Enter the full path** to the log file you want to monitor (e.g., `/var/log/syslog`, `tests/logs_for_testing/dummy_log1.txt`, `tests/logs_for_testing/Apache_2k.log`, or `tests/logs_for_testing/dns_with_timestamps.log`).
4. **Click the "Add" or "Watch" button** to start monitoring the log file.
5. The log file will appear in the dashboard, and new entries will be displayed in real time.

> **Tip:**  
> You can add more than one log at a time by entering multiple log paths, one per line and then click on button Add Path(s).  
> Make sure the log file exists and is readable by the application.

---

## Testing

- Example log files for testing are available in `tests/logs_for_testing/`:
    - `dummy_log1.txt` – Example line-based log file
    - `json1.json` – Example JSON log file
    - `Apache_2k.log` – Example Apache log file
    - `dns_with_timestamps.log` – Example DNS log file with timestamps
- Use `generator.sh` in the `tests/` folder to generate or modify test logs.

---

## Usage

- Add log files to watch via the dashboard.
- Set filters or notifications for specific patterns (line-based logs only).
- Search logs using string or regex queries (works for both line-based and JSON logs).
- JSON logs are displayed in a pretty-printed format and only support search.

---

## Contributing

Pull requests and issues are welcome!

---

## License

MIT License