searchState.loadedDescShard("cas", 0, "Create the app: setup everything and returns a <code>Router</code>\nApp config\nHost URL\nDatabase URL for Postgres\nToken used by Expo API to send a notification\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nJWT secret used for key creation\nA new configuration read from the env\nLevel of Rust logging\nSetup database connection. Get variable <code>DATABASE_URL</code> from …\nAll errors raised by the web app\nGeneric bad request. It is handled with a message value\nDatabase error\nRaised when a passed token is not valid\nNot found error\nRaised when a token is not good created\nRaised if an user wants to do something can’t do\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMatches <code>AppError</code> into a tuple of status and error message. …\nConnection to an Expo client\nSend notifications using Expo\nSetup a new Expo API\nMutation struct\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMake GraphQL login\nMake GraphQL request to create new alert. Only for admins.\nMake GraphQL request to create new position to track\nMake GraphQL call to register a notification device token …\nQuery struct\nReturns all the positions\nReturns the API version. It is like a “greet” function\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns all the last positions for each user. It is …\nReturns all the notifications. They can be filtered by an …\nReturns all the positions\nReturns an user by ID. Admins can check everyone.\nReturns all the users. It is restricted to admins only.\nHandler for GraphQL route. It executs the schema using the …\nAlert struct\nAlert input struct\nEnumeration which refers to the level of alert\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGet alerts from the database\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nBody used as response to login\nAuthentication enum\nClaims struct.\nAccess token string\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the token as a string. If a token is not encoded, …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreate a new Claim using the <code>user_id</code> and the current …\n“Bearer” string\nUser id\nNotification struct\nReturns the argument unchanged.\nGet notifications from the database\nCalls <code>U::from(self)</code>.\nCreate a new notification into the database from an …\nEnumeration which refers to the kind of moving activity\nPosition struct\nPosition input struct\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGet positions from the database\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nGet last positions from the database for each user. It is …\nUser struct\nFind an user with id = <code>id</code> using the PostgreSQL <code>client</code>\nReturns the argument unchanged.\nReturns the argument unchanged.\nGet users from the database\nGet users from the database\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nSetup tracing subscriber logger\nExtension of <code>Json</code> which returns the CREATED status code\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nState application shared through Axum\nPostgreSQL client synced via Arc\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.")