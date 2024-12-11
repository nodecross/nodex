import json
from platform_os import is_windows


if is_windows():
    from request import post
else:
    from sock import post


payload = {
    "message": {
        "binary_url": "https://github.com/nodecross/nodex/releases/download/v3.3.0/nodex-agent-x86_64.zip",
        "path": "/tmp/nodex",
    }
}

json_response = post("/internal/version/update", payload)

print("The response is as follows.\n")
print(json_response)
