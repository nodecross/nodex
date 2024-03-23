from sock import post


def main():
    json_response = post("/identifiers")

    print("The response is as follows.\n")
    print(json_response)


if __name__ == "__main__":
    main()
