## aio
a simple io using io_uring

### samples
```
cargo build --release
and run ./hello
then push a request `http://127.0.0.1:12345` on browser
you will get the result as follows
```
![](./imgs/io_uring_http.png)

### load test
```
oha -n 1000000 -c 1000 -q 50000 --latency-correction --disable-keepalive http://127.0.0.1:12345
```
![](./imgs/io_uring.gif)

```
Summary:
  Success rate:	100.00%
  Total:	20.0148 secs
  Slowest:	0.0220 secs
  Fastest:	0.0001 secs
  Average:	0.0012 secs
  Requests/sec:	4996.3035

  Total data:	15.83 MiB
  Size/request:	166 B
  Size/sec:	809.95 KiB

Response time histogram:
  0.000 [1]     |
  0.002 [89448] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.004 [8290]  |■■
  0.007 [1461]  |
  0.009 [401]   |
  0.011 [181]   |
  0.013 [160]   |
  0.015 [32]    |
  0.018 [4]     |
  0.020 [12]    |
  0.022 [10]    |


Response time distribution:
  10.00% in 0.0005 secs
  25.00% in 0.0006 secs
  50.00% in 0.0009 secs
  75.00% in 0.0014 secs
  90.00% in 0.0023 secs
  95.00% in 0.0033 secs
  99.00% in 0.0061 secs
  99.90% in 0.0123 secs
  99.99% in 0.0197 secs


Details (average, fastest, slowest):
  DNS+dialup:	0.0004 secs, 0.0000 secs, 0.0193 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0023 secs

Status code distribution:
  [200] 100000 responses

```
