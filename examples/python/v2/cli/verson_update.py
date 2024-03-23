from sock import post


def main():
    payload = {
        "binary_url": "https://example.com/nodex-agent-1.0.0.zip",
        "path": "/tmp",
    }

    json_response = post("/internal/version/update", payload)

    print("The response is as follows.\n")
    print(json_response)


if __name__ == "__main__":
    main()
