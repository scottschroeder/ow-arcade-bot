#!/bin/bash

curl -L -v https://overwatcharcade.today/api/v1/overwatch/today | jq . > example_today_api_call.json
