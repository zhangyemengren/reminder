import {useEffect, useRef, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import "./App.css";

const TimeStatus = {
    Paused: 0,
    Running: 1,
    Finished: 2,
};

const sendNotification = (title, body) => invoke("send_notification", {title, body});
const getStoreValue = (key) => invoke("get_store_value", {key});
const setStoreValue = (key, value) => invoke("set_store_value", {key, value});
const startTimeTask = (seconds) => invoke("start_time_task", {seconds});
const pauseTimeTask = (key) => invoke("pause_time_task", {key});
const resumeTimeTask = (key) => invoke("resume_time_task", {key});
const stopTimeTask = (key) => invoke("stop_time_task", {key});

const generateKey = () => `local_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;

const formatTime = (seconds) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
};

function App() {
    const [tasks, setTasks] = useState([]);
    const [loading, setLoading] = useState(true);
    const backendKeyMapRef = useRef({});

    // 通用更新任务函数
    const updateTask = (localKey, changes) => {
        setTasks((prev) => prev.map((t) => (t.localKey === localKey ? {...t, ...changes} : t)));
    };

    // 保存到 store
    const saveTasks = async (tasksToSave) => {
        await setStoreValue("countList", tasksToSave.map(({name, seconds, localKey, loop}) => ({name, seconds, localKey, loop})));
    };

    // 初始化
    useEffect(() => {
        getStoreValue("countList")
            .then((res) => {
                setTasks((res ?? []).map((item) => ({
                    name: item.name || "未命名任务",
                    seconds: item.seconds || 60,
                    localKey: item.localKey || generateKey(),
                    remainingSeconds: item.seconds || 60,
                    processing: false,
                    paused: false,
                    editing: false,
                    loop: item.loop || false,
                })));
            })
            .catch((e) => console.error("加载数据失败:", e))
            .finally(() => setLoading(false));
    }, []);

    // 用 ref 存储最新的 tasks，供监听器访问
    const tasksRef = useRef([]);
    useEffect(() => {
        tasksRef.current = tasks;
    }, [tasks]);

    // 事件监听
    useEffect(() => {
        const unlistenPromise = listen("time-tick", async (event) => {
            const {key: backendKey, seconds, status} = event.payload;
            const localKey = backendKeyMapRef.current[backendKey];
            if (!localKey) return;

            if (status === TimeStatus.Finished) {
                // 先删除映射，防止重复处理
                delete backendKeyMapRef.current[backendKey];

                // 从 ref 获取最新的任务信息
                const task = tasksRef.current.find((t) => t.localKey === localKey);
                if (!task) return;

                sendNotification("倒计时完成", `任务「${task.name}」已完成！`);

                if (task.loop) {
                    // 先重置状态为非运行（在新任务创建前不能暂停）
                    setTasks((prev) =>
                        prev.map((t) =>
                            t.localKey === localKey
                                ? {...t, remainingSeconds: task.seconds, processing: false, paused: false}
                                : t
                        )
                    );

                    // 立即启动新任务
                    try {
                        const newBackendKey = await startTimeTask(task.seconds);
                        backendKeyMapRef.current[newBackendKey] = localKey;
                        setTasks((prev) =>
                            prev.map((t) =>
                                t.localKey === localKey
                                    ? {...t, processing: true, paused: false}
                                    : t
                            )
                        );
                    } catch (e) {
                        console.error("自动重启倒计时失败:", e);
                    }
                } else {
                    // 非循环任务完成后直接重置
                    setTasks((prev) =>
                        prev.map((t) =>
                            t.localKey === localKey
                                ? {...t, remainingSeconds: task.seconds, processing: false, paused: false}
                                : t
                        )
                    );
                }
                return;
            }

            // 非 Finished 状态
            setTasks((prev) =>
                prev.map((t) =>
                    t.localKey === localKey
                        ? {...t, remainingSeconds: seconds, processing: status === TimeStatus.Running, paused: status === TimeStatus.Paused}
                        : t
                )
            );
        });

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, []);

    const handleStart = async (localKey) => {
        const task = tasks.find((t) => t.localKey === localKey);
        if (!task || task.processing) return;

        try {
            const backendKey = await startTimeTask(task.remainingSeconds);
            backendKeyMapRef.current[backendKey] = localKey;
            updateTask(localKey, {processing: true, paused: false});
        } catch (e) {
            console.error("启动倒计时失败:", e);
        }
    };

    const getBackendKey = (localKey) => {
        return Object.entries(backendKeyMapRef.current).find(([, lk]) => lk === localKey)?.[0];
    };

    const handlePause = async (localKey) => {
        const backendKey = getBackendKey(localKey);
        if (!backendKey) return;

        try {
            await pauseTimeTask(backendKey);
            updateTask(localKey, {processing: false, paused: true});
        } catch (e) {
            console.error("暂停倒计时失败:", e);
        }
    };

    const handleResume = async (localKey) => {
        const backendKey = getBackendKey(localKey);
        if (!backendKey) return;

        try {
            await resumeTimeTask(backendKey);
            updateTask(localKey, {processing: true, paused: false});
        } catch (e) {
            console.error("继续倒计时失败:", e);
        }
    };

    const handleStop = async (localKey) => {
        const task = tasks.find((t) => t.localKey === localKey);
        if (!task) return;

        const backendKey = getBackendKey(localKey);
        if (backendKey) {
            try {
                await stopTimeTask(backendKey);
                delete backendKeyMapRef.current[backendKey];
            } catch (e) {
                console.error("终止倒计时失败:", e);
            }
        }
        updateTask(localKey, {remainingSeconds: task.seconds, processing: false, paused: false});
    };

    const handleSave = async (localKey, newName, newSeconds) => {
        const updatedTasks = tasks.map((t) =>
            t.localKey === localKey
                ? {...t, name: newName, seconds: newSeconds, remainingSeconds: newSeconds, editing: false, processing: false}
                : t
        );
        setTasks(updatedTasks);
        await saveTasks(updatedTasks);
    };

    const handleAddTask = async () => {
        const newTask = {
            name: "新任务",
            seconds: 60,
            localKey: generateKey(),
            remainingSeconds: 60,
            processing: false,
            paused: false,
            editing: true,
            loop: false,
        };
        const updatedTasks = [...tasks, newTask];
        setTasks(updatedTasks);
        await saveTasks(updatedTasks);
    };

    const handleToggleLoop = async (localKey) => {
        const updatedTasks = tasks.map((t) => (t.localKey === localKey ? {...t, loop: !t.loop} : t));
        setTasks(updatedTasks);
        await saveTasks(updatedTasks);
    };

    const handleDeleteTask = async (localKey) => {
        // 清理 backendKeyMap 中的映射
        Object.entries(backendKeyMapRef.current).forEach(([bk, lk]) => {
            if (lk === localKey) delete backendKeyMapRef.current[bk];
        });
        const updatedTasks = tasks.filter((t) => t.localKey !== localKey);
        setTasks(updatedTasks);
        await saveTasks(updatedTasks);
    };

    if (loading) {
        return (
            <main className="container">
                <div className="loading">
                    <div className="loading-spinner"></div>
                    <span>加载中...</span>
                </div>
            </main>
        );
    }

    return (
        <main className="container">
            <header className="app-header">
                <h1>倒计时任务 v0.1.7</h1>
                <span className="task-count">{tasks.length} 个任务</span>
            </header>

            <div className="task-list">
                {tasks.length === 0 ? (
                    <div className="empty-state">
                        <div className="empty-icon">⏱</div>
                        <p>还没有任务</p>
                        <p className="empty-hint">点击下方按钮添加第一个任务</p>
                    </div>
                ) : (
                    tasks.map((task) => (
                        <TaskItem
                            key={task.localKey}
                            task={task}
                            onStart={() => handleStart(task.localKey)}
                            onPause={() => handlePause(task.localKey)}
                            onResume={() => handleResume(task.localKey)}
                            onStop={() => handleStop(task.localKey)}
                            onEdit={() => updateTask(task.localKey, {editing: true})}
                            onSave={(name, seconds) => handleSave(task.localKey, name, seconds)}
                            onCancel={() => updateTask(task.localKey, {editing: false})}
                            onDelete={() => handleDeleteTask(task.localKey)}
                            onToggleLoop={() => handleToggleLoop(task.localKey)}
                        />
                    ))
                )}
            </div>

            <button className="add-btn" onClick={handleAddTask} title="添加新任务">
                <span className="add-icon">+</span>
            </button>
        </main>
    );
}

function TaskItem({task, onStart, onPause, onResume, onStop, onEdit, onSave, onCancel, onDelete, onToggleLoop}) {
    const [editName, setEditName] = useState(task.name);
    const [editMinutes, setEditMinutes] = useState(Math.floor(task.seconds / 60));
    const [editSecs, setEditSecs] = useState(task.seconds % 60);

    useEffect(() => {
        if (task.editing) {
            setEditName(task.name);
            setEditMinutes(Math.floor(task.seconds / 60));
            setEditSecs(task.seconds % 60);
        }
    }, [task.editing, task.name, task.seconds]);

    const handleSaveClick = () => {
        const totalSeconds = editMinutes * 60 + editSecs;
        if (totalSeconds > 0) onSave(editName || "未命名任务", totalSeconds);
    };

    const progress = task.seconds > 0 ? ((task.seconds - task.remainingSeconds) / task.seconds) * 100 : 0;

    if (task.editing) {
        return (
            <div className="task-item editing">
                <div className="edit-form">
                    <div className="edit-row">
                        <label>任务名称</label>
                        <input
                            type="text"
                            value={editName}
                            onChange={(e) => setEditName(e.target.value)}
                            placeholder="输入任务名称"
                            className="task-name-input"
                            autoFocus
                        />
                    </div>
                    <div className="edit-row">
                        <label>倒计时时长</label>
                        <div className="time-inputs">
                            <div className="time-input-group">
                                <input
                                    type="number"
                                    value={editMinutes}
                                    onChange={(e) => setEditMinutes(Math.max(0, parseInt(e.target.value) || 0))}
                                    className="time-input"
                                    min="0"
                                    max="999"
                                />
                                <span className="time-label">分</span>
                            </div>
                            <div className="time-input-group">
                                <input
                                    type="number"
                                    value={editSecs}
                                    onChange={(e) => setEditSecs(Math.max(0, Math.min(59, parseInt(e.target.value) || 0)))}
                                    className="time-input"
                                    min="0"
                                    max="59"
                                />
                                <span className="time-label">秒</span>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="task-actions">
                    <button className="btn btn-primary" onClick={handleSaveClick}>保存</button>
                    <button className="btn btn-secondary" onClick={onCancel}>取消</button>
                    <button className="btn btn-danger" onClick={onDelete}>删除</button>
                </div>
            </div>
        );
    }

    return (
        <div className={`task-item ${task.processing ? "running" : ""} ${task.paused ? "paused" : ""}`}>
            <div className="task-progress" style={{width: `${progress}%`}}></div>
            <div className="task-content">
                <div className="task-info">
                    <span className="task-name">{task.name}</span>
                    {task.loop && <span className="status-badge loop">循环</span>}
                    {task.processing && <span className="status-badge running">运行中</span>}
                    {task.paused && <span className="status-badge paused">已暂停</span>}
                </div>
                <span className="task-time">
                    {formatTime(task.remainingSeconds)}
                </span>
            </div>
            <div className="task-actions">
                <button
                    className={`btn btn-loop ${task.loop ? "active" : ""}`}
                    onClick={onToggleLoop}
                    title={task.loop ? "关闭循环" : "开启循环"}
                    disabled={task.processing || task.paused}
                >
                    循环
                </button>
                {task.processing ? (
                    <>
                        <button className="btn btn-warning" onClick={onPause}>暂停</button>
                        <button className="btn btn-danger" onClick={onStop}>终止</button>
                    </>
                ) : task.paused ? (
                    <>
                        <button className="btn btn-primary" onClick={onResume}>继续</button>
                        <button className="btn btn-danger" onClick={onStop}>终止</button>
                    </>
                ) : (
                    <button className="btn btn-primary" onClick={onStart}>开始</button>
                )}
                <button className="btn btn-secondary" onClick={onEdit} disabled={task.processing || task.paused}>编辑</button>
            </div>
        </div>
    );
}

export default App;
