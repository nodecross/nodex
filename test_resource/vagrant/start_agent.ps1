$ENV:NODEX_DID_HTTP_ENDPOINT="https://192.168.56.3:4010"
$ENV:NODEX_DID_ATTACHMENT_LINK="https://192.168.56.3:4010"
$ENV:NODEX_STUDIO_HTTP_ENDPOINT="https://192.168.56.3:4011"
$ENV:HTTPS_PROXY="https://192.168.56.3:3129"
$ENV:NODEX_SERVER_PORT="3000"

$agentPath = Join-Path -Path $PWD -ChildPath ".\nodex\target\debug\nodex-agent.exe"
$stdErrLogTmp = Join-Path -Path $PWD -ChildPath ".\stderr.log"
$stdOutLogTmp = Join-Path -Path $PWD -ChildPath ".\stdout.log"
Start-Process -FilePath "$agentPath" -NoNewWindow -RedirectStandardOutput "$stdOutLogTmp" -RedirectStandardError "$stdErrLogTmp"
Start-Sleep -Seconds 10
cd nodex\e2e
cargo test
