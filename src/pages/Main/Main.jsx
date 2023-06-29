import { useForm } from "react-hook-form"

import styles from "./Main.module.css"
import { invoke } from "@tauri-apps/api"
import { useEffect, useState } from "react"

const Main = () => {
  const [profiles, setProfiles] = useState([])

  const {
    register,
    handleSubmit,
    formState: {
      errors
    },
  } = useForm()
  const onSubmit = (data) => console.log(data)

  useEffect(() => {
    invoke("get_profiles").then((resp) => setProfiles(resp))
  }, [])


  return (
    <form onSubmit={handleSubmit(onSubmit)} className={styles.form}>
      <input type="text" {...register("username", {
        required: true,
        minLength: 3,
        maxLength: 16,
        pattern: /^[a-zA-Z0-9_]{3,16}$/mg
      })}
      className={styles.input}
      />
      <div className={styles.select}>
        {/* TODO: Add a customizable select */}
        <select {...register("profile")}>
          {
            profiles.map((option) => (
              <option value={option.id} key={option.id}>
                {option.name}
              </option>
            ))
          }
        </select>
      </div>

      {errors?.username && <div>
        <span>Requirements:</span>
          <ul>
            <li>
              Needs to consist of 3-16 characters
            </li>
            <li>
              No spaces
            </li>
          </ul>

          <span>Allowed characters:</span>
          <ul>
            <li>
              A-Z (upper and lower case)
            </li>
            <li>
              0-9
            </li>
            <li>
              The only allowed special character is _ (underscore)
            </li>
          </ul>
        </div>}

      <input type="submit" className={styles.button} value="Launch" />
    </form>
  )
}

export default Main