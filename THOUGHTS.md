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
