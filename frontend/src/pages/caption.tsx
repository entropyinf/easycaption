import {listen} from "@tauri-apps/api/event";
import {useEffect, useState} from "react";

export default function Caption() {
    const [caption, setCaption] = useState('')

    useEffect(() => {
        try {
            listen<Caption>('caption', (event) => {
                try {
                    const caption = event.payload
                    setCaption(caption.text)
                } catch (error) {
                    console.log(error)
                }
            });
        } catch (error) {
            console.log(error)
        }
    }, [])

    return (
        <div
            className="h-full bg-black/50 text-white p-4 rounded-md flex flex-col absolute bottom-0 left-0 right-0 select-none"
            data-tauri-drag-region>
            <h1 className="text-3xl font-medium text-center select-none">
                {caption}
            </h1>
        </div>
    )

}

type Caption = {
    text: string
    start: number
    end: number
}