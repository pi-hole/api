# Pi-hole API

Work in progress HTTP API for Pi-hole.
The API reads FTL's shared memory so it can directly read the statistics FTL
generates. This API is the replacement for most of FTL's socket/telnet API, as
well as the PHP API of the pre-5.0 web interface.