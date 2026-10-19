
import Contacts
import Foundation

let store = CNContactStore()
let fileManager = FileManager.default

let addressBookRoot =
    NSHomeDirectory() + "/Library/Application Support/AddressBook/Sources"

func containerTypeName(_ type: CNContainerType) -> String {
    switch type {
    case .local:
        return "local"
    case .exchange:
        return "exchange"
    case .cardDAV:
        return "cardDAV"
    case .unassigned:
        return "unassigned"
    @unknown default:
        return "unknown"
    }
}

do {
    let containers = try store.containers(matching: nil)

    let results: [[String: Any]] = containers.map { container in
        let components = container.identifier.components(separatedBy: ":")
        let candidateUUID = components.first ?? ""

        let hasExpectedFormat =
            components.count == 2 &&
            components[1] == "ABAccount" &&
            UUID(uuidString: candidateUUID) != nil

        let sourceUUID: String? =
            hasExpectedFormat ? candidateUUID : nil

        let sourcePath: String? = sourceUUID.map {
            (addressBookRoot as NSString)
                .appendingPathComponent($0)
        }

        var isDirectory: ObjCBool = false
        let directoryExists: Bool

        if let path = sourcePath {
            directoryExists = fileManager.fileExists(
                atPath: path,
                isDirectory: &isDirectory
            ) && isDirectory.boolValue
        } else {
            directoryExists = false
        }

        return [
            "name": container.name,
            "identifier": container.identifier,
            "type": container.type.rawValue,
            "typeName": containerTypeName(container.type),
            "sourceUUID": sourceUUID as Any? ?? NSNull(),
            "sourcePath": sourcePath as Any? ?? NSNull(),
            "sourceDirectoryExists": directoryExists,
            "isICloudCandidate":
                container.name == "iCloud" &&
                container.type == .cardDAV &&
                sourceUUID != nil &&
                directoryExists
        ]
    }

    let output: [String: Any] = [
        "addressBookRoot": addressBookRoot,
        "containers": results
    ]

    let data = try JSONSerialization.data(
        withJSONObject: output,
        options: [.prettyPrinted, .sortedKeys]
    )

    print(String(decoding: data, as: UTF8.self))

} catch {
    fputs(
        "Error enumerating Contacts containers: \(error)\n",
        stderr
    )
    exit(1)
}
