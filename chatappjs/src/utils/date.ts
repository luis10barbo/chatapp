export function parseDataDB(dataDB: String) {
  return new Date(dataDB.replace(" ", "T") + "Z");
}
