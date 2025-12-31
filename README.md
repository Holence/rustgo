- 交替落子
  - 不可下入“禁着点”
  - 若使周边的对方棋子气尽，则提吃
- 地多为胜

禁着点:
  - 禁止下到已有的棋子上
  - 禁止使己方气尽
  - 禁止全局同形

# 代码设计

Client-Server架构

Server
- Board: 中国规则的围棋棋盘
- Game: 棋局, 调度多个Player (使用协程? TODO)

- Player接口
  - 可以对接 GTP in child process (pipe) 的 GoEngine
  - 可以对接 GTP in tcp 的 远程Client
  - 可以对接 内置的来自前端的用户交互
  - 可以对接 内置的 GoEngine

# TODO

- GTP with gnugo
- learn cfg
  - use feature Conditional compilation
