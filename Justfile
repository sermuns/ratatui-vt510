virtual:
	socat -d -d pty,raw,echo=0 pty,raw,echo=0

client:
	picocom /dev/pts/5 -b 115200 --quiet --noinit

host:
	cargo r -- /dev/pts/4
