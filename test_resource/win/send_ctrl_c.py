import ctypes
import time
import psutil
import sys

kernel32 = ctypes.windll.kernel32
CTRL_C_EVENT = 0

def send_ctrl_c(pid):
    if not kernel32.AttachConsole(pid):
        print(f"Error attaching to console: {ctypes.GetLastError()}")
        return
    kernel32.SetConsoleCtrlHandler(None, True)

    if not kernel32.GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0):
        print(f"Error generating CTRL+C event: {ctypes.GetLastError()}")
        kernel32.FreeConsole()
        return

    time.sleep(1)
    kernel32.FreeConsole()
    kernel32.SetConsoleCtrlHandler(None, False)

def find_and_send_ctrl_c(process_name):
    for proc in psutil.process_iter(['pid', 'name']):
        if proc.info['name'] == process_name:
            print(f"Sending CTRL+C to {process_name} with PID: {proc.info['pid']}")
            send_ctrl_c(proc.info['pid'])

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python send_ctrl_c.py <process_name>")
    else:
        process_name = sys.argv[1]
        find_and_send_ctrl_c(process_name)
