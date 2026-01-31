# LoggyXP Testing Guide

This guide describes how to test LoggyXP using the provided test logs and scripts.

---

## Test Files & Utilities

- **Test logs:**  
  Located in `tests/logs_for_testing/`  
  - `dummy_log1.txt` – Example line-based log file  
  - `json1.json` – Example JSON log file

- **Log generator script:**  
  `tests/generator.sh`  
  This script generates and appends lines to `/tmp/dummyLogs/demo.txt` and `/tmp/dummyLogs/demo2.txt` for live tailing tests.

---

## Manual Test Cases

### 1. **Basic Log Watching**
- **Action:** Add `tests/logs_for_testing/dummy_log1.txt` via the dashboard.
- **Expected:** The log appears in the dashboard and new lines appended to the file are shown in real time.

### 2. **Live Tailing with Generator**
- **Action:** Run `tests/generator.sh 2` in a terminal to generate logs every 2 seconds.
- **Action:** Add `/tmp/dummyLogs/demo.txt` and `/tmp/dummyLogs/demo2.txt` via the dashboard.
- **Expected:** New lines appear in the dashboard as they are written.

### 3. **Multiple Log Files**
- **Action:** Add both `/tmp/dummyLogs/demo.txt` and `/tmp/dummyLogs/demo2.txt` at once.
- **Expected:** Both logs are tailed independently and updates are shown for each.

### 4. **Filtering (Line-based logs only)**
- **Action:** Set a filter pattern (e.g., "Counting file 1") for `/tmp/dummyLogs/demo.txt`.
- **Expected:** Only lines containing "Counting file 1" are displayed for that log.

### 5. **Notification (Line-based logs only)**
- **Action:** Set a notification pattern (e.g., "file 2") for `/tmp/dummyLogs/demo2.txt`.
- **Expected:** When a line containing "file 2" is appended, a notification is triggered in the dashboard.

### 6. **Search (String and Regex)**
- **Action:** Use the search feature to find "Counting" in `/tmp/dummyLogs/demo.txt`.
- **Expected:** All lines containing "Counting" are returned.

- **Action:** Use a regex search (e.g., `file \d`) in `/tmp/dummyLogs/demo2.txt`.
- **Expected:** All lines matching the pattern are returned.

### 7. **JSON Log Support**
- **Action:** Add `tests/logs_for_testing/json1.json` as a log file.
- **Expected:** The JSON is displayed in a pretty-printed format. Tailing, filtering, and notifications are **not** available. Only search works.

- **Action:** Perform a search for a key or value present in `json1.json`.
- **Expected:** Matching lines are returned.

### 8. **Batch Log Sending**
- **Action:** Add a large log file (over 200 lines).
- **Expected:** Log lines are sent and displayed in batches (not all at once).

### 9. **Permissions and Error Handling**
- **Action:** Try to add a non-existent or unreadable log file.
- **Expected:** An error message is shown and the log is not added.

---

## Automated Testing Suggestions

- Use `tests/generator.sh` to continuously append lines and verify real-time updates in the dashboard.
- Write integration tests using a WebSocket client (Python, Rust, etc.) to simulate dashboard actions and verify server responses.

---

## Example: Using `generator.sh`

```sh
cd tests
./generator.sh 1
```
- This will append lines to `/tmp/dummyLogs/demo.txt` and `/tmp/dummyLogs/demo2.txt` every 1 second.
- Add these files in the dashboard to see live updates.

---

## Notes

- Only line-based logs support tailing, filtering, and notifications.
- JSON logs support only display and search.
- Make sure the application has read permissions for the log files.

---

