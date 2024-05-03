from request import post


payload = {
    "message": {
        "binary_url": "https://example.com/nodex-agent-1.0.0.zip",
        "path": "C:\\Temp",
    }
}


json_response = post("/internal/version/update", payload)

print("The response is as follows.\n")
print(json_response)
