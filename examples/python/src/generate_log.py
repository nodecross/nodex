import time
from platform_os import is_windows

if is_windows():
    from request import post
else:
    from sock import post

log = {
    "message": "test-message",
    "occurred_at": int(time.time() * 1000),
}

json_response = post("/logs", log)

print("The response is as follows.\n")
print(json_response)
