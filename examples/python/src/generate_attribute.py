from sock import post


json_response = post(
    "/attributes",
    {
        "key_name": "test-key-name",
        "value": "test-value",
    },
)

print("The response is as follows.\n")
print(json_response)
