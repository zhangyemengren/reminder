import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import "./App.css";

function App() {
    const [count, setCount] = useState(10);

    async function sendNotification() {
        await invoke("send_notification", {title: "Hello", body: "World"});
    }
    async function startTimer() {
        await invoke("start_time_task", {seconds: count});
    }

    useEffect(() => {
        let unsubscribe;
        listen("time-tick", (event) => {
            setCount(event.payload);
        }).then((fn) => {
            unsubscribe = fn;
        });

        return () => {
            unsubscribe?.();
        };
    }, []);

    return (
        <main className="container">
            <button onClick={sendNotification}>Send Notification</button>
            <button onClick={startTimer}>Start Timer</button>
            <div>
                <div>喝水倒计时</div>
                <div>{count}秒</div>
            </div>
        </main>
    );
}

export default App;
