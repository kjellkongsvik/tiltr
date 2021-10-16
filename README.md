Listen for a tilt sample.

Might be used to send data to brewfather:
```bash
curl -d `tiltr -t 10` -H "Content-Type: application/json" -X POST http://log.brewfather.net/stream?id=123456
```
