#  LoggyXP Architecture

![Alt text](screenshots/LoggyXP_Architecture_Diagram.svg) 

## Components description

### WebUI

* Shows log information in real time and offline mode.
* Control panel for filtering, searching and monitoring.

### WebServer

* API for communication between LogMgr and Web UI.
* Individual threads for each monitoring.

### LogMgr

* Application manager.
* Fulfills the role of interpreter, communicator, commander and overall manager for LogFiltering, SearchEngine, LogVisualizer, LogMonitoring and Notification software components.

### LogFiltering

* This component will receive data defined and custom patterns introduced by the user to filter data in one or more logs. 
* Log parser.
* Real time tail filtering.

### SearchEngine

* Handle searching through log/logs.
* Regex integration

### LogVisualizer

* Log formatting
* Combining results from the other components for visualizing.
* It acts as the logger of the whole application.

### LogMonitoring

* File watching.
* Async IO.
* Handling multiple logs from the system storage.

### Notification

* Prioritize alerts.
* Set alerts for specific patterns
* Filter alerts.