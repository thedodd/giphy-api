giphy api
=========
Design and implement a web application that allows a user to search for and save animated GIFs to a user profile using the [GIPHY API](https://developers.giphy.com/docs/).

**Requirements:**
- Provide users with the ability to register and login to your application.
- As well as the ability to save and view their favorite GIFs to their profile.
- Also provide the user with the ability to categorize these saved GIFs(ex: funny, animals, etc.).
- User data should be stored in a database.
- Basic application security practices should be implemented (OWASP Top 9).
- GIFs available on your application should be limited to a G-Rating.

### overview
#### api
The API is implemented as a [Protocol Buffers](https://developers.google.com/protocol-buffers/docs/overview) based Websockets API.

All interaction with this API is performed over Websockets and must adhere to the protocol outlined in the API's main protobuf file.
