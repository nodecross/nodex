import json
from platform_os import is_windows


if is_windows():
    from request import post
else:
    from sock import post


payload = {
    "message": {
        "binary_url": "http://localhost:9000/nodex-agent.zip",
        "path": "/tmp/nodex2",
    }
}

json_response = post("/internal/version/update", payload)

print("The response is as follows.\n")
print(json_response)
