package actions

import io.gatling.javaapi.core.CoreDsl.StringBody
import io.gatling.javaapi.core.ScenarioBuilder
import io.gatling.javaapi.http.HttpDsl.http

fun ScenarioBuilder.createAccommodation() =
    exec(
        http("Create accommodation")
            .post("/graphql")
            .body(
                StringBody(
                    """
                    {
                        "query": "mutation {\n  addAccommodation(input: {\n    name: \"Hotel 1\"\n    description: \"This is a luxury hotel.\"\n    address: {\n      street: \"Sample street\"\n      houseNumber: 1\n      zipCode: \"12345\"\n      city: \"Stuttgart\"\n      country: DE\n    }\n  }) {\n    id\n    name\n  }\n}"
                    }
                    """
                        .trimIndent()))
            .asJson())
