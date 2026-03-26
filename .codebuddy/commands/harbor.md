将刚才的上面需求转化成一个agent任务
产物的格式参考下面的文档：
https://harborframework.com/docs/tasks

格式参考下面的：
 task_name/                
  ├── log/trajectory.jsonl    # 可以问Codebuddy我们对话的日志放在哪里？把日志上传到task文件夹中
  ├── instruction.md          # 任务说明 — agent 看到的唯一输入                                                                               
  ├── task.toml               # 任务配置 — 资源、超时、环境变量                                                                                      
  ├── environment/             
  │   └── Dockerfile          # 运行环境 — agent 和 verifier 共用的容器
  ├── solution/
  │   └── solve.sh            # 参考解法 — Oracle agent 用，证明任务可解
  └── tests/
      ├── test.sh             # 验证入口 — Harbor 调用的唯一入口脚本
      ├── test_dialect_recording.py  # API 测试
      └── eval_agent.py              # Computer Use 视觉评估

将此次对话的日志文件拷贝到 task_name 目录下的 log目录中
