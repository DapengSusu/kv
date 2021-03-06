# kv

## 架构和设计

#### 流程示意图
![image](https://user-images.githubusercontent.com/37730928/138604118-2ffe382c-01c9-4f26-aded-5a4a8fee9eb7.png)

#### 需要思考的问题
1. 客户端和服务器用什么协议通信？TCP？gRPC？HTTP？支持一种还是多种？
2. 客户端和服务器之间交互的应用层协议如何定义？怎么做序列化 / 反序列化？是用 Protobuf、JSON 还是 Redis RESP？或者也可以支持多种？
3. 服务器都支持哪些命令？第一版优先支持哪些？
4. 具体的处理逻辑中，需不需要加 hook，在处理过程中发布一些事件，让其他流程可以得到通知，进行额外的处理？这些 hook 可不可以提前终止整个流程的处理？
5. 对于存储，要支持不同的存储引擎么？比如 MemDb（内存）、RocksDb（磁盘）、SledDb（磁盘）等。对于 MemDb，我们考虑支持 WAL（Write-Ahead Log） 和 snapshot 么？
6. 整个系统可以配置么？比如服务使用哪个端口、哪个存储引擎？
7. …

#### 设计思路（参考）
1. 像 KV Server 这样需要高性能的场景，通信应该优先考虑 TCP 协议。所以我们暂时只支持 TCP，未来可以根据需要支持更多的协议，如 HTTP2/gRPC。还有，未来可能对安全性有额外的要求，所以我们要保证 TLS 这样的安全协议可以即插即用。总之，网络层需要灵活。
2. 应用层协议我们可以用 protobuf 定义。protobuf 直接解决了协议的定义以及如何序列化和反序列化。Redis 的 RESP 固然不错，但它的短板也显而易见，命令需要额外的解析，而且大量的 \r\n 来分隔命令或者数据，也有些浪费带宽。使用 JSON 的话更加浪费带宽，且 JSON 的解析效率不高，尤其是数据量很大的时候。protobuf 就很适合 KV server 这样的场景，灵活、可向后兼容式升级、解析效率很高、生成的二进制非常省带宽，唯一的缺点是需要额外的工具 protoc 来编译成不同的语言。虽然 protobuf 是首选，但也许未来为了和 Redis 客户端互通，还是要支持 RESP。
3. 服务器支持的命令我们可以参考Redis 的命令集。第一版先来支持 HXXX 命令，比如 HSET、HMSET、HGET、HMGET 等。从命令到命令的响应，可以做个 trait 来抽象。
4. 处理流程中计划加这些 hook：收到客户端的命令后 OnRequestReceived、处理完客户端的命令后 OnRequestExecuted、发送响应之前 BeforeResponseSend、发送响应之后 AfterResponseSend。这样，处理过程中的主要步骤都有事件暴露出去，让我们的 KV server 可以非常灵活，方便调用者在初始化服务的时候注入额外的处理逻辑。
5. 存储必然需要足够灵活。可以对存储做个 trait 来抽象其基本的行为，一开始可以就只做 MemDb，未来肯定需要有支持持久化的存储。
6. 需要支持配置，但优先级不高。等基本流程搞定，使用过程中发现足够的痛点，就可以考虑配置文件如何处理了。

#### 重要接口 
最重要的几个接口就是三个主体交互的接口：
* 客户端和服务器的接口或者说协议
* 服务器和命令处理流程的接口
* 服务器和存储的接口。

## 使用方法
1. git clone git@github.com:DapengSusu/kv.git
2. cd kv
3. ~~export BUILD_PROTO=1~~
4. cargo build --release
5. cargo test
6. cargo r --release --example service --quiet
7. 另开一个终端: cargo r --release --example client --quiet

## 下一步计划
* 为剩下 6 个命令 HMGET、HMSET、HDEL、HMDEL、HEXIST、HMEXIST 构建测试，并实现它们
* 实现 MemTable 的 get_iter() 方法
* 延伸：可以创建一个线程池，每个线程有自己的 HashMap。当 HGET/HSET 等命令来临时，可以对 key 做个哈希，然后分派到 “拥有” 那个 key 的线程，这样，可以避免在处理的时候加锁，提高系统的吞吐

##### 实现剩余命令
- [ ] HMGET
- [ ] HMSET
- [x] HDEL
- [ ] HMDEL
- [ ] HEXIST
- [ ] HMEXIST
- [ ] ...
