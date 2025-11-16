# LoggyXP

Project designed in RUST to monitor multiple log files simultaneously, 
parses different formats, allowing you to search and filter based on patterns.

In the future it will include a web dashboard for visualization.

### How to run it:
You must have a rust compiler installed to build it and then proceed to execute the binary.

## Important
This code is just a proof of concept, it has basic monitor, search and filter functions
with hardcoded values.

The code uses a dummy log for testing and demonstration purposes. 
log is located in directory:

**loggyxp\tests\logs_for_testing\dummy_log1.txt**

The log currently contains a poem by Emily Dickinson

As the state of sw is now, does the following
* Retrieves dummy_log1.txt
* Prints it on the console
* Looks for the patter 'Nobody' and how many times it appears in the file.
* Filter the lines that contains pattern 'Nobody'.



