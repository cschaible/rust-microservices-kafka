package actions

import io.gatling.javaapi.core.CoreDsl.StringBody
import io.gatling.javaapi.core.ScenarioBuilder
import io.gatling.javaapi.http.HttpDsl.http
import java.util.UUID.randomUUID

fun ScenarioBuilder.createUser() =
    exec(
        http("Create user")
            .post("/users")
            .body(
                StringBody(
                    """
                    {
                      "name": "${randomUUID()}",
                      "email": "${randomUUID()}@example.com",
                      "country": "US",
                      "phoneNumbers": []
                    }
                    """
                        .trimIndent()))
            .asJson())
