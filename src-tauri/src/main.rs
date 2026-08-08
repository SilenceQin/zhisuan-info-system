// 防止 Windows 调试时弹出 console 窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    zhisuan_data_app_lib::run()
}
