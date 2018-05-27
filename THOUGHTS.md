Use Rust for perf, Yoga for layout. Attempt to add zero typing, scrolling
latency.

Use Ctrl-Ctrl double tap to get into command mode. Ctrl-\[any other key\] will
automatically clear the double-tap, so it doesn't add latency.

MOST IMPORTANT THING: active pane paint events happen quickly. Throttle all
other paint events if neccessary. Track how much you're sending to the screen
at once and throttle the entire thing's paint events if necessary, even the
active pane. You must simulate an entire VT100 terminal here: draw your own
layout, and send to the screen YOUR REPRESENTATION OF THAT LAYOUT: don't send
output to the controlling terminal 1:1.

Use dirty rects. Painting is character-by-character and is slow.

Have a timer that forces you to paint at 60fps (still using dirty rects for
perf). Throttle updates to inactive panes to 20fps.

How to do client/server stuff:

* In the server, create a unix socket server at a known location.
* In the client, create a pair of Unix domain sockets with
  `UnixStream::pair()`.
* Have the client connect to the server's known socket and send one of the
  paired sockets over the stream.
* In the server, spawn a thread to handle the new socket that was sent over the
  stream.
* In the client, send the stdin, stdout, stderr streams over the paired socket.
* In the server, grab the streams sent over the socket and use them for
  in/out/err.
* In the server, if the paired socket ever shuts down, shut down the thread.

This has a bit of overhead to spawn a new client, but zero overhead for reading
or writing data. This alone should beat tmux's perf on macOS and WSL.
