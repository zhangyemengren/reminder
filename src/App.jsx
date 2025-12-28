import {useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";

function App() {
    const [greetMsg, setGreetMsg] = useState("");
    const [name, setName] = useState("");

    async function sendNotification() {
        await invoke("send_notification", {title: "Hello", body: "World"});
    }

    return (
        <main className="container">
            <button onClick={sendNotification}>Send Notification</button>
            <div>
                <div>喝水倒计时</div>
                <div>10分钟</div>
            </div>
            <div>
                <div>撒尿倒计时</div>
                <div>10分钟</div>
            </div>
        </main>
    );
}

export default App;
