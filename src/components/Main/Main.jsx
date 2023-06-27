import { useForm } from "react-hook-form"

import styles from "./Main.module.css"

const Main = () => {
  const { register, handleSubmit } = useForm()
  const onSubmit = (data) => console.log(data)

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <input type="text" {...register("username")}/>

      <input type="submit" />
    </form>
  )
}

export default Main