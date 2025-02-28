#!/usr/bin/env nu

# run this with `nu path/to/this/file`

use std assert

# Import the function to test
use open_file.nu is_hx_running

# Define test cases
def test_cases [] {
    [
        # Basic cases
        ["hx", true, "Basic 'hx' command"],
        ["HX", true, "Uppercase 'HX'"],
        ["hx ", true, "hx with trailing space"],
        [" hx", true, "hx with leading space"],
        
        # Path cases
        ["/hx", true, "hx at root"],
        ["/usr/local/bin/hx", true, "Full path to hx"],
        ["./hx", true, "Relative path to hx"],
        ["../hx", true, "Parent directory hx"],
        ["some/path/to/hx", true, "Nested path to hx"],
        
        # With arguments
        ["hx .", true, "hx with current directory"],
        ["hx file.txt", true, "hx with file argument"],
        ["hx -c theme:base16", true, "hx with flag"],
        ["hx --help", true, "hx with long flag"],
        ["/usr/local/bin/hx --version", true, "Full path hx with flag"],
        
        # Negative cases
        ["vim", false, "Different editor"],
        ["echo hx", false, "hx in echo command"],
        ["path/with/hx/in/middle", false, "hx in middle of path"],
        ["hx_file", false, "hx as part of filename"],
    ]
}

# Run tests
def run_tests [] {
    mut passed_count = 0
    let n_tests = test_cases | length 
    
    for case in (test_cases) {
        let input = $case.0
        let expected = $case.1
        let description = $case.2
        
        print $"Testing: ($description)"
        let result = (is_hx_running $input)
        assert equal $result $expected $"Failed: ($description) - Expected ($expected), got ($result)"
        
        # If the assertion passes, increment the counter and print the number of passed tests
        $passed_count = $passed_count + 1
        print $"Passed test #($passed_count) of ($n_tests): ($description)"
        print ""
    }
    
    print $"Total tests passed: ($passed_count)"
}

# Main test runner
def main [] {
    print "Running tests for is_hx_running function..."
    run_tests
    print "All tests completed!"
}
