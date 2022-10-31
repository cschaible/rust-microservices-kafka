package config

import io.gatling.javaapi.http.HttpDsl
import kotlin.time.Duration
import kotlin.time.toJavaDuration

class Configuration(port: Int) {

  val baseUrl = "http://localhost:$port"
  val concurrentUsers = 10
  val rampUpDuration = Duration.parse("15s").toJavaDuration()
  val testDuration = Duration.parse("1m").toJavaDuration()

  val httpProtocol =
      HttpDsl.http
          .baseUrl(baseUrl)
          .inferHtmlResources()
          .acceptHeader("application/json")
          .acceptEncodingHeader("gzip, deflate")
          .acceptLanguageHeader("en-US")
          .contentTypeHeader("application/json")
}
