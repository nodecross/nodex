import * as os from "os";
import * as path from "path";
import got from "got";

export const base = `unix:${path.join(os.homedir(), ".nodex/run/nodex.sock")}`;

const call = async (method, path, json) => {
  let response;
  const URL = [base, path].join(":");
  console.log(`calling ${method} ${URL}`);
  switch (method) {
    case "get":
      response = await got
        .get(URL, {
          enableUnixSockets: true,
        })
        .json();
      break;
    case "post":
      response = await got
        .post(URL, {
          enableUnixSockets: true,
          json,
        })
        .json();
      break;
    default:
      throw new Error(`Unsupported method: ${method}`);
  }
  return JSON.stringify(response, null, 4);
};

export const get = async (path) => call("get", path, null);
export const post = async (path, json) => call("post", path, json);
