package main

import (
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"
)

func main() {
	// Define and parse command line arguments
	sleepTime := flag.Int("sleepTime", 5, "amount of time it will run / sleep before exiting")
	signalNum := flag.Int("signal", 15, "expected signals to cleanly stop")
	waitTime := flag.Int("waitTime", 5, "amount of waiting time when entering the clean stopping")
	exitCode := flag.Int("exitCode", 0, "the exitcode it returns")
	flag.Parse()

	// Exit directly if no argument is specified
	if flag.NFlag() == 0 {
		fmt.Fprintln(os.Stderr, "No arguments specified. Exiting...")
		os.Exit(0)
	}

	// Log current working directory
	wd, _ := os.Getwd()
	fmt.Fprintln(os.Stderr, "Current working directory:", wd)

	// Print environment variables
	fmt.Fprintln(os.Stdout, "Environment variables:")
	for _, env := range os.Environ() {
		fmt.Fprintln(os.Stdout, env)
	}

	// Print umask in octal form
	fmt.Fprintf(os.Stdout, "Umask: 0o%03o\n", syscall.Umask(0))
	syscall.Umask(syscall.Umask(0))

	// Handle signals and clean stopping
	signalChan := make(chan os.Signal, 1)
	signal.Notify(signalChan, syscall.Signal(*signalNum))

	go func() {
		<-signalChan
		fmt.Fprintln(os.Stderr, "Received signal. Waiting before clean stop...")
		time.Sleep(time.Duration(*waitTime) * time.Second)
		os.Exit(*exitCode)
	}()

	// Sleep for the specified amount of time
	time.Sleep(time.Duration(*sleepTime) * time.Second)

	// Ensure the program does not exit before the wait time has finished
	if *waitTime > *sleepTime {
		remainingWaitTime := *waitTime - *sleepTime
		time.Sleep(time.Duration(remainingWaitTime) * time.Second)
	}

	os.Exit(*exitCode)
}
