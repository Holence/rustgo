- 交替落子
  - 不可下入“禁着点”
    - 禁止下到已有的棋子上
      - 禁止使己方气尽
      - 禁止全局同形
  - 若使周边的对方棋子气尽，则提吃
- 地多为胜

# 代码设计

Client-Server架构

Server
- Board: 中国规则的围棋棋盘
- Game: 对局控制器, 使用协程调度多个Player
- Player接口
  - 可以对接 GTP in child process (pipe) 的 GoEngine
  - 可以对接 GTP in tcp 的 远程Client
  - 可以对接 内置的来自前端的用户交互
  - 可以对接 内置的 GoEngine

# 阶段目标

- [x] Board 中国规则棋盘，支持多色棋
- [x] Game 控制器，支持Player组队，可以拉着本地的狗组队对战
- [x] 上tokio
- [x] `Server = Game + Vec<TeamHandle<Vec<PlayerHandle>>>`，用channel传递ServerMessage/PlayerMessage
- [x] 简易的 egui 界面，可以展示多色棋组队对战效果
- [ ] Server搭建Actor Model，模拟PlayerMessage流入Lobby/Room/Game后的处理
- [ ] GUI Client实现Lobby/Room/Game
- [ ] Server支持TCP连接
- [ ] ...
- [ ] HTTP+WebSocket???

# TODO

- 不要全部用usize, usize不是固定的, 定义
  - 抽离disjoint_set，固定其Idx类型
  - board中的idx直接用usize，方便计算
  - type stone_nums
- learn cfg
  - use feature Conditional compilation
- TUI
- 削减pub
- 聊天中 Bot名字+闭嘴
- http+DB (信息CRUD) + websocket (对战Actor Model)
  - server axum
  - client reqwest+tokio-tungstenite?
