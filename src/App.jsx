import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {listen} from '@tauri-apps/api/event';
import "./App.css";

function App() {
    const [count, setCount] = useState(5);
    const [storeText, setStoreText] = useState("");

    async function sendNotification() {
        await invoke("send_notification", {title: "Hello", body: "World"});
    }
    async function startTimer() {
        await invoke("start_time_task", {seconds: count});
    }
    async function getStoreValue() {
        return await invoke("get_store_value", {key: "count"});
    }
    async function setStoreValue(text) {
        await invoke("set_store_value", {key: "text", value: text});
        setStoreText(text);
    }


    useEffect(() => {
        setStoreValue("Hello").then(response => console.log(response)).catch(error => console.error(error));
    }, []);

    useEffect(() => {
        let unsubscribe;
        invoke("get_event_key").then((key) => {
            return listen(key, (event) => {
                setCount(event.payload);
                if (event.payload === 0) {
                    sendNotification()
                }
            })
        }).then((fn) => {
            unsubscribe = fn;
        });

        return () => {
            unsubscribe?.();
        };
    }, []);

    return (
        <main className="container">
            <button onClick={startTimer}>Start Timer</button>
            <button onClick={() => getStoreValue().then(setStoreText)}>Get Text</button>
            <div>
                <div>store: {storeText}</div>
            </div>
            <div>
                <div>喝水倒计时</div>
                <div>{count}秒</div>
            </div>
        </main>
    );
}

export default App;
