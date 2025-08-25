
export default function Caption() {
    return (
        <div className="h-full bg-black/50 text-white p-4 rounded-md flex flex-col absolute bottom-0 left-0 right-0 select-none" >
            <h1 className="text-3xl font-medium text-center select-none" data-tauri-drag-region>
                字幕内容
            </h1>
        </div>
    )

}