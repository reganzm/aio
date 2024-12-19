## aio
a simple io using epoll
### use case
```
run hello.rs
the server listened on 127.0.0.1:12345
on browser push a request `http://127.0.0.1:12345`
you will get the result hello.html as follows
```
![](./imgs/reactor_epoll.png)

### code structure
![](./imgs/reactor_epoll_structure.png)