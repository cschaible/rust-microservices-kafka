import io.gatling.recorder.GatlingRecorder
import io.gatling.recorder.config.RecorderPropertiesBuilder
import io.gatling.recorder.render.template.Format
import scala.Option

object Recorder {

  @JvmStatic
  fun main(args: Array<String>) {
    GatlingRecorder.fromMap(
        RecorderPropertiesBuilder()
            .simulationsFolder(IDEPathHelper.gradleSourcesDirectory.toString())
            .resourcesFolder(IDEPathHelper.gradleResourcesDirectory.toString())
            .simulationFormat(Format.fromString("kotkin"))
            .certificatePath(
                IDEPathHelper.gradleResourcesDirectory
                    .resolve("cert")
                    .resolve("gatling.cert.pem")
                    .toString())
            .privateKeyPath(
                IDEPathHelper.gradleResourcesDirectory
                    .resolve("cert")
                    .resolve("gatling.key.pem")
                    .toString())
            .httpsMode("CertificateAuthority")
            .simulationPackage("simulations")
            .denyList(
                listOf(
                    ".*\\.css",
                    ".*\\.gif",
                    ".*\\.ico",
                    ".*\\.js",
                    ".*\\.jpeg",
                    ".*\\.jpg",
                    ".*\\.json",
                    ".*\\.png",
                    ".*\\.svg",
                    ".*\\.woff",
                    ".*\\.woff2",
                    ".*\\.(t|o)tf"))
            .localPort(8080)
            .build(),
        Option.apply(IDEPathHelper.recorderConfigFile))
  }
}
